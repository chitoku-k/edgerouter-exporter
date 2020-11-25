package cmd

import (
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/chitoku-k/edgerouter-exporter/service"
	"github.com/sirupsen/logrus"
)

var (
	groupRegexp = regexp.MustCompile(`^Group (.+)`)
	itemRegexp  = regexp.MustCompile(`(.+): (.+)`)
	runRegexp   = regexp.MustCompile(`(\d+)/(\d+)`)
	pingRegexp  = regexp.MustCompile(`^(.+) - (.+)`)
)

type Line int

const (
	LineNone Line = iota
	LineGroup
	LineInterface
	LineItem
)

type Parser interface {
	ParseLoadBalancerWatchdog(data []string) ([]service.LoadBalanceGroup, error)
}

type parser struct {
}

func NewParser() Parser {
	return &parser{}
}

func (p *parser) ParseLoadBalancerWatchdog(data []string) ([]service.LoadBalanceGroup, error) {
	var previous Line
	var result []service.LoadBalanceGroup

	var current *service.LoadBalanceGroup
	for _, line := range data {
		if len(strings.TrimSpace(line)) == 0 {
			previous = LineNone
			continue
		}

		if itemRegexp.MatchString(line) {
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

func parseInt(key, value string) int {
	n, err := strconv.ParseInt(value, 10, strconv.IntSize)
	if err != nil {
		logrus.Infof(`Cannot parse "%s" to an integer (key "%s"): %v`, value, key, err)
	}
	return int(n)
}

func parseTime(key, value string) *time.Time {
	t, err := time.Parse("Mon Jan 02 15:04:05 2006", value)
	if err != nil {
		logrus.Infof(`Cannot parse "%s" to a time (key: "%s"): %v`, value, key, err)
		return nil
	}
	return &t
}
