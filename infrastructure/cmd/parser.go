package cmd

import (
	"fmt"
	"net"
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/alecthomas/units"
	"github.com/chitoku-k/edgerouter-exporter/service"
	"github.com/dannav/hhmmss"
	"github.com/sirupsen/logrus"
	str2duration "github.com/xhit/go-str2duration/v2"
)

var (
	noActiveRegexp      = regexp.MustCompile(`^No active .*sessions`)
	notConfiguredRegexp = regexp.MustCompile(`.* not configured`)
	bgpRouterRegexp     = regexp.MustCompile(`BGP router identifier ([\d.]+), local AS number (\d+)`)
	bgpTableRegexp      = regexp.MustCompile(`BGP table version is (\d+)`)
	bgpASPathRegexp     = regexp.MustCompile(`(\d+) BGP AS-PATH entries`)
	bgpCommunityRegexp  = regexp.MustCompile(`(\d+) BGP community entries`)
	separatorRegexp     = regexp.MustCompile(`^[ -]+$`)
	interfaceRegexp     = regexp.MustCompile(`([^\[]+)`)
	groupRegexp         = regexp.MustCompile(`^Group (.+)`)
	itemRegexp          = regexp.MustCompile(`(.+?): (.+)`)
	runRegexp           = regexp.MustCompile(`(\d+)/(\d+)`)
	pingRegexp          = regexp.MustCompile(`^(.+) - (.+)`)

	byteUnits = units.MakeUnitMap("", "", 1024)
)

type Line int

const (
	LineNone Line = iota
	LineBGPStatus
	LineGroup
	LineInterface
	LineSeparator
	LineItem
)

type TimeLayout string

const (
	GenericTimeLayout TimeLayout = "Mon Jan _2 15:04:05 2006"
	BuildTimeLayout   TimeLayout = "01/02/06 15:04"
)

type Parser interface {
	ParseVersion(data []string) (service.Version, error)
	ParseBGPStatus(data []string, protocol service.IPProtocol) (service.BGPStatus, error)
	ParseDdnsStatus(data []string) ([]service.DdnsStatus, error)
	ParseLoadBalanceWatchdog(data []string) ([]service.LoadBalanceGroup, error)
	ParsePPPoEClientSessions(data []string) ([]service.PPPoEClientSession, error)
}

type parser struct {
}

func NewParser() Parser {
	return &parser{}
}

func (p *parser) ParseVersion(data []string) (service.Version, error) {
	var result service.Version

	for _, line := range data {
		if !itemRegexp.MatchString(line) {
			continue
		}

		m := itemRegexp.FindStringSubmatch(line)
		key := strings.TrimSpace(m[1])
		value := strings.TrimSpace(m[2])

		switch key {
		case "Version":
			result.Version = value

		case "Build ID":
			result.BuildID = value

		case "Build on":
			result.BuildOn = parseTime(BuildTimeLayout, key, value)

		case "Copyright":
			result.Copyright = value

		case "HW model":
			result.HWModel = value

		case "HW S/N":
			result.HWSerialNumber = value

		case "Uptime":
			result.Uptime = value
		}
	}

	if result.Version == "" {
		return result, fmt.Errorf("expected version, found nothing")
	}

	return result, nil
}

func (p *parser) ParseBGPStatus(data []string, protocol service.IPProtocol) (service.BGPStatus, error) {
	var previous Line
	var result service.BGPStatus

	for _, line := range data {
		if len(strings.TrimSpace(line)) == 0 {
			previous = LineNone
			continue
		}

		if bgpRouterRegexp.MatchString(line) {
			m := bgpRouterRegexp.FindStringSubmatch(line)
			result.RouterID = strings.TrimSpace(m[1])
			result.LocalAS = parseInt("local AS number", m[2])
			previous = LineBGPStatus
			continue
		}

		if bgpTableRegexp.MatchString(line) {
			m := bgpTableRegexp.FindStringSubmatch(line)
			result.TableVersion = parseInt("table version", m[1])
			previous = LineBGPStatus
			continue
		}

		if bgpASPathRegexp.MatchString(line) {
			m := bgpASPathRegexp.FindStringSubmatch(line)
			result.ASPaths = parseInt("AS-PATH entries", m[1])
			previous = LineBGPStatus
			continue
		}

		if bgpCommunityRegexp.MatchString(line) {
			m := bgpCommunityRegexp.FindStringSubmatch(line)
			result.Communities = parseInt("community entries", m[1])
			previous = LineBGPStatus
			continue
		}

		if previous == LineNone {
			previous = LineSeparator
			continue
		}

		if previous == LineSeparator || previous == LineItem {
			previous = LineItem
			fields := strings.Fields(line)
			if len(fields) != 10 {
				return result, fmt.Errorf("unexpected number of fields, expecting 10 fields: %v", line)
			}

			addr := net.ParseIP(fields[0])
			switch {
			case addr == nil:
				return result, fmt.Errorf("failed to parse BGP neighbor %q", addr)

			case addr.To4() == nil && protocol == service.IPv4:
				continue

			case addr.To4() != nil && protocol == service.IPv6:
				continue
			}

			var uptime *time.Duration
			if strings.Contains(fields[8], ":") {
				uptime = parseDuration("Up/Down", fields[8])
			}
			if strings.Contains(fields[8], "d") {
				uptime = parseDurationUnit("Up/Down", fields[8])
			}

			var state string
			pfxrcd, err := strconv.ParseInt(fields[9], 10, strconv.IntSize)
			if err != nil {
				state = fields[9]
			}

			result.Neighbors = append(result.Neighbors, service.BGPNeighbor{
				Address:          addr.To16(),
				Version:          parseInt("V", fields[1]),
				RemoteAS:         parseInt("AS", fields[2]),
				MessagesReceived: parseInt64("MsgRcv", fields[3]),
				MessagesSent:     parseInt64("MsgSen", fields[4]),
				TableVersion:     parseInt("TblVer", fields[5]),
				InQueue:          parseInt64("InQ", fields[6]),
				OutQueue:         parseInt64("OutQ", fields[7]),
				Uptime:           uptime,
				State:            state,
				PrefixesReceived: pfxrcd,
			})
			continue
		}
	}

	return result, nil
}

func (p *parser) ParseDdnsStatus(data []string) ([]service.DdnsStatus, error) {
	var previous Line
	var result []service.DdnsStatus

	var current *service.DdnsStatus
	for _, line := range data {
		if len(strings.TrimSpace(line)) == 0 {
			previous = LineNone
			continue
		}

		if notConfiguredRegexp.MatchString(line) {
			return nil, nil
		}

		if previous == LineNone {
			if current != nil {
				result = append(result, *current)
			}
			current = &service.DdnsStatus{}
		}

		if itemRegexp.MatchString(line) {
			previous = LineItem
			m := itemRegexp.FindStringSubmatch(line)
			key := strings.TrimSpace(m[1])
			value := strings.TrimSpace(m[2])

			switch key {
			case "interface":
				m = interfaceRegexp.FindStringSubmatch(value)
				current.Interface = strings.TrimSpace(m[1])

			case "ip address":
				current.IPAddress = value

			case "host-name":
				current.HostName = value

			case "last update":
				current.LastUpdate = parseTime(GenericTimeLayout, key, value)

			case "update-status":
				current.UpdateStatus = value
			}
		}
	}

	if current != nil {
		result = append(result, *current)
	}

	return result, nil
}

func (p *parser) ParseLoadBalanceWatchdog(data []string) ([]service.LoadBalanceGroup, error) {
	var previous Line
	var result []service.LoadBalanceGroup

	var current *service.LoadBalanceGroup
	for _, line := range data {
		if len(strings.TrimSpace(line)) == 0 {
			previous = LineNone
			continue
		}

		if notConfiguredRegexp.MatchString(line) {
			return nil, nil
		}

		if itemRegexp.MatchString(line) {
			if current == nil {
				return nil, fmt.Errorf("unexpected token, expecting group: %v", line)
			}
			if len(current.Statuses) == 0 {
				return nil, fmt.Errorf("unexpected token, expecting group or empty line: %v", line)
			}

			previous = LineItem
			m := itemRegexp.FindStringSubmatch(line)
			status := &current.Statuses[len(current.Statuses)-1]
			key := strings.TrimSpace(m[1])
			value := strings.TrimSpace(m[2])

			switch key {
			case "status":
				status.Status = value

			case "pings":
				status.Pings = parseInt(key, value)

			case "fails":
				status.Fails = parseInt(key, value)

			case "run fails":
				m = runRegexp.FindStringSubmatch(value)
				status.RunFails = parseInt(key, m[1])

			case "route drops":
				status.RouteDrops = parseInt(key, value)

			case "ping gateway":
				m = pingRegexp.FindStringSubmatch(value)
				status.Ping = service.LoadBalancePing{
					Gateway: strings.TrimSpace(m[1]),
					Status:  strings.TrimSpace(m[2]),
				}

			case "last route drop":
				status.LastRouteDrop = parseTime(GenericTimeLayout, key, value)

			case "last route recover":
				status.LastRouteRecover = parseTime(GenericTimeLayout, key, value)
			}

			continue
		}

		if groupRegexp.MatchString(line) {
			previous = LineGroup
			m := groupRegexp.FindStringSubmatch(line)
			name := strings.TrimSpace(m[1])

			if current != nil {
				result = append(result, *current)
			}
			current = &service.LoadBalanceGroup{Name: name}
			continue
		}

		if previous == LineGroup || previous == LineNone {
			previous = LineInterface
			current.Statuses = append(current.Statuses, service.LoadBalanceStatus{
				Interface: strings.TrimSpace(line),
			})
			continue
		}
	}

	if current != nil {
		result = append(result, *current)
	}

	return result, nil
}

func (p *parser) ParsePPPoEClientSessions(data []string) ([]service.PPPoEClientSession, error) {
	var previous Line
	var result []service.PPPoEClientSession

	var current *service.PPPoEClientSession
	for _, line := range data {
		if len(strings.TrimSpace(line)) == 0 {
			previous = LineNone
			continue
		}

		if noActiveRegexp.MatchString(line) {
			return nil, nil
		}

		if separatorRegexp.MatchString(line) {
			previous = LineSeparator
			continue
		}

		if previous == LineSeparator || previous == LineItem {
			previous = LineItem
			fields := strings.Fields(line)
			if len(fields) != 9 {
				return result, fmt.Errorf("unexpected number of fields, expecting 9 fields: %v", line)
			}

			if current != nil {
				result = append(result, *current)
			}
			current = &service.PPPoEClientSession{
				User:            fields[0],
				Time:            parseDurationUnit("User", fields[1]),
				Protocol:        fields[2],
				Interface:       fields[3],
				RemoteIP:        fields[4],
				TransmitPackets: parseBytes("TX pkt", fields[5]),
				TransmitBytes:   parseBytes("TX byte", fields[6]),
				ReceivePackets:  parseBytes("RX pkt", fields[7]),
				ReceiveBytes:    parseBytes("RX byte", fields[8]),
			}
		}
	}

	if current != nil {
		result = append(result, *current)
	}

	return result, nil
}

func parseInt(key, value string) int {
	return int(parseInt64(key, value))
}

func parseInt64(key, value string) int64 {
	n, err := strconv.ParseInt(value, 10, strconv.IntSize)
	if err != nil {
		logrus.Infof(`Cannot parse %q to an integer (key %q): %v`, value, key, err)
	}
	return n
}

func parseBytes(key, value string) int64 {
	u, err := units.ParseUnit(value, byteUnits)
	if err != nil {
		logrus.Infof(`Cannot parse %q to a byte unit (key: %q): %v`, value, key, err)
	}
	return u
}

func parseTime(layout TimeLayout, key, value string) *time.Time {
	t, err := time.Parse(string(layout), value)
	if err != nil {
		logrus.Infof(`Cannot parse %q to a time (key: %q): %v`, value, key, err)
		return nil
	}
	return &t
}

func parseDuration(key, value string) *time.Duration {
	d, err := hhmmss.Parse(value)
	if err != nil {
		logrus.Infof(`Cannot parse %q to a duration (key: %q): %v`, value, key, err)
		return nil
	}
	return &d
}

func parseDurationUnit(key, value string) *time.Duration {
	d, err := str2duration.ParseDuration(value)
	if err != nil {
		logrus.Infof(`Cannot parse %q to a duration (key: %q): %v`, value, key, err)
		return nil
	}
	return &d
}
