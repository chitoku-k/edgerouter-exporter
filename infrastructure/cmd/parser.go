package cmd

import (
	"fmt"
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/alecthomas/units"
	"github.com/chitoku-k/edgerouter-exporter/service"
	"github.com/sirupsen/logrus"
	str2duration "github.com/xhit/go-str2duration/v2"
)

var (
	noActiveRegexp      = regexp.MustCompile(`^No active .*sessions`)
	notConfiguredRegexp = regexp.MustCompile(`.* not configured`)
	separatorRegexp     = regexp.MustCompile(`^[ -]+$`)
	interfaceRegexp     = regexp.MustCompile(`([^\[]+)`)
	groupRegexp         = regexp.MustCompile(`^Group (.+)`)
	itemRegexp          = regexp.MustCompile(`(.+): (.+)`)
	runRegexp           = regexp.MustCompile(`(\d+)/(\d+)`)
	pingRegexp          = regexp.MustCompile(`^(.+) - (.+)`)

	byteUnits = units.MakeUnitMap("", "", 1024)
)

type Line int

const (
	LineNone Line = iota
	LineGroup
	LineInterface
	LineSeparator
	LineItem
)

type Parser interface {
	ParseDdnsStatus(data []string) ([]service.DdnsStatus, error)
	ParseLoadBalanceWatchdog(data []string) ([]service.LoadBalanceGroup, error)
	ParsePPPoEClientSessions(data []string) ([]service.PPPoEClientSession, error)
}

type parser struct {
}

func NewParser() Parser {
	return &parser{}
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
				current.LastUpdate = parseTime(key, value)

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
				status.LastRouteDrop = parseTime(key, value)

			case "last route recover":
				status.LastRouteRecover = parseTime(key, value)
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
				Time:            parseDuration("[1]", fields[1]),
				Protocol:        fields[2],
				Interface:       fields[3],
				RemoteIP:        fields[4],
				TransmitPackets: parseBytes("[5]", fields[5]),
				TransmitBytes:   parseBytes("[6]", fields[6]),
				ReceivePackets:  parseBytes("[7]", fields[7]),
				ReceiveBytes:    parseBytes("[8]", fields[8]),
			}
		}
	}

	if current != nil {
		result = append(result, *current)
	}

	return result, nil
}

func parseInt(key, value string) int {
	n, err := strconv.ParseInt(value, 10, strconv.IntSize)
	if err != nil {
		logrus.Infof(`Cannot parse "%s" to an integer (key "%s"): %v`, value, key, err)
	}
	return int(n)
}

func parseBytes(key, value string) int64 {
	u, err := units.ParseUnit(value, byteUnits)
	if err != nil {
		logrus.Infof(`Cannot parse "%s" to a byte unit (key: "%s"): %v`, value, key, err)
	}
	return u
}

func parseTime(key, value string) *time.Time {
	t, err := time.Parse("Mon Jan _2 15:04:05 2006", value)
	if err != nil {
		logrus.Infof(`Cannot parse "%s" to a time (key: "%s"): %v`, value, key, err)
		return nil
	}
	return &t
}

func parseDuration(key, value string) *time.Duration {
	d, err := str2duration.ParseDuration(value)
	if err != nil {
		logrus.Infof(`Cannot parse "%s" to a duration (key: "%s"): %v`, value, key, err)
		return nil
	}
	return &d
}
