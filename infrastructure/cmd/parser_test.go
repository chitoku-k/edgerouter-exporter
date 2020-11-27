package cmd_test

import (
	"time"

	"github.com/chitoku-k/edgerouter-exporter/infrastructure/cmd"
	"github.com/chitoku-k/edgerouter-exporter/service"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("Parser", func() {
	var (
		parser cmd.Parser
	)

	BeforeEach(func() {
		parser = cmd.NewParser()
	})

	Describe("ParseDdnsStatus()", func() {
		Context("when empty data is given", func() {
			It("returns empty", func() {
				actual, err := parser.ParseDdnsStatus(nil)
				Expect(actual).To(BeEmpty())
				Expect(err).NotTo(HaveOccurred())
			})
		})

		Context("when one status is given", func() {
			It("returns one group", func() {
				actual, err := parser.ParseDdnsStatus([]string{
					"interface    : eth0",
					"ip address   : 192.0.2.1",
					"host-name    : example.com",
					"last update  : Mon Jan 02 15:04:05 2006",
					"update-status: good",
					"",
				})
				Expect(actual).To(Equal([]service.DdnsStatus{
					{
						Interface:    "eth0",
						IPAddress:    "192.0.2.1",
						HostName:     "example.com",
						LastUpdate:   parseTime("2006-01-02T15:04:05Z"),
						UpdateStatus: "good",
					},
				}))
				Expect(err).NotTo(HaveOccurred())
			})
		})

		Context("when multiple statuses are given", func() {
			It("returns multiple statuses", func() {
				actual, err := parser.ParseDdnsStatus([]string{
					"interface    : eth0",
					"ip address   : 192.0.2.1",
					"host-name    : 1.example.com",
					"last update  : Mon Jan 02 15:04:05 2006",
					"update-status: good",
					"",
					"interface    : eth1",
					"ip address   : 192.0.2.2",
					"host-name    : 2.example.com",
					"last update  : Mon Jan 02 15:04:06 2006",
					"update-status: good",
					"",
				})
				Expect(actual).To(Equal([]service.DdnsStatus{
					{
						Interface:    "eth0",
						IPAddress:    "192.0.2.1",
						HostName:     "1.example.com",
						LastUpdate:   parseTime("2006-01-02T15:04:05Z"),
						UpdateStatus: "good",
					},
					{
						Interface:    "eth1",
						IPAddress:    "192.0.2.2",
						HostName:     "2.example.com",
						LastUpdate:   parseTime("2006-01-02T15:04:06Z"),
						UpdateStatus: "good",
					},
				}))
				Expect(err).NotTo(HaveOccurred())
			})
		})
	})

	Describe("ParseLoadBalanceWatchdog()", func() {
		Context("when empty data is given", func() {
			It("returns empty", func() {
				actual, err := parser.ParseLoadBalanceWatchdog(nil)
				Expect(actual).To(BeEmpty())
				Expect(err).NotTo(HaveOccurred())
			})
		})

		Context("when one group is given", func() {
			It("returns one group", func() {
				actual, err := parser.ParseLoadBalanceWatchdog([]string{
					"Group FAILOVER_01",
					"  eth0",
					"  status: OK",
					"  pings: 7777",
					"  fails: 1",
					"  run fails: 0/3",
					"  route drops: 0",
					"  ping gateway: ping.ubnt.com - REACHABLE",
					"",
					"  eth1",
					"  status: Waiting on recovery (0/3)",
					"  pings: 7777",
					"  fails: 77",
					"  run fails: 3/3",
					"  route drops: 1",
					"  ping gateway: ping.ubnt.com - DOWN",
					"  last route drop   : Mon Jan 02 15:04:05 2006",
					"  last route recover: Mon Jan 02 15:04:00 2006",
					"",
				})
				Expect(actual).To(Equal([]service.LoadBalanceGroup{
					{
						Name: "FAILOVER_01",
						Statuses: []service.LoadBalanceStatus{
							{
								Interface:        "eth0",
								Status:           "OK",
								FailoverOnlyMode: false,
								Pings:            7777,
								Fails:            1,
								RunFails:         0,
								RouteDrops:       0,
								Ping: service.LoadBalancePing{
									Gateway: "ping.ubnt.com",
									Status:  "REACHABLE",
								},
								LastRouteDrop:    nil,
								LastRouteRecover: nil,
							},
							{
								Interface:        "eth1",
								Status:           "Waiting on recovery (0/3)",
								FailoverOnlyMode: false,
								Pings:            7777,
								Fails:            77,
								RunFails:         3,
								RouteDrops:       1,
								Ping: service.LoadBalancePing{
									Gateway: "ping.ubnt.com",
									Status:  "DOWN",
								},
								LastRouteDrop:    parseTime("2006-01-02T15:04:05Z"),
								LastRouteRecover: parseTime("2006-01-02T15:04:00Z"),
							},
						},
					},
				}))
				Expect(err).NotTo(HaveOccurred())
			})
		})

		Context("when multiple groups are given", func() {
			It("returns all groups", func() {
				actual, err := parser.ParseLoadBalanceWatchdog([]string{
					"Group FAILOVER_01",
					"  eth0",
					"  status: OK",
					"  pings: 1000",
					"  fails: 1",
					"  run fails: 0/3",
					"  route drops: 0",
					"  ping gateway: ping.ubnt.com - REACHABLE",
					"",
					"  eth1",
					"  status: Waiting on recovery (0/3)",
					"  pings: 1000",
					"  fails: 10",
					"  run fails: 3/3",
					"  route drops: 1",
					"  ping gateway: ping.ubnt.com - DOWN",
					"  last route drop   : Mon Jan 02 15:04:05 2006",
					"  last route recover: Mon Jan 02 15:04:00 2006",
					"",
					"Group FAILOVER_02",
					"  eth2",
					"  status: OK",
					"  pings: 1000",
					"  fails: 0",
					"  run fails: 0/3",
					"  route drops: 0",
					"  ping gateway: ping.ubnt.com - REACHABLE",
					"",
					"  eth3",
					"  status: OK",
					"  pings: 1000",
					"  fails: 0",
					"  run fails: 0/3",
					"  route drops: 0",
					"  ping gateway: ping.ubnt.com - REACHABLE",
					"  last route drop   : Mon Jan 02 15:04:05 2006",
					"",
					"",
				})
				Expect(actual).To(Equal([]service.LoadBalanceGroup{
					{
						Name: "FAILOVER_01",
						Statuses: []service.LoadBalanceStatus{
							{
								Interface:        "eth0",
								Status:           "OK",
								FailoverOnlyMode: false,
								Pings:            1000,
								Fails:            1,
								RunFails:         0,
								RouteDrops:       0,
								Ping: service.LoadBalancePing{
									Gateway: "ping.ubnt.com",
									Status:  "REACHABLE",
								},
								LastRouteDrop:    nil,
								LastRouteRecover: nil,
							},
							{
								Interface:        "eth1",
								Status:           "Waiting on recovery (0/3)",
								FailoverOnlyMode: false,
								Pings:            1000,
								Fails:            10,
								RunFails:         3,
								RouteDrops:       1,
								Ping: service.LoadBalancePing{
									Gateway: "ping.ubnt.com",
									Status:  "DOWN",
								},
								LastRouteDrop:    parseTime("2006-01-02T15:04:05Z"),
								LastRouteRecover: parseTime("2006-01-02T15:04:00Z"),
							},
						},
					},
					{
						Name: "FAILOVER_02",
						Statuses: []service.LoadBalanceStatus{
							{
								Interface:        "eth2",
								Status:           "OK",
								FailoverOnlyMode: false,
								Pings:            1000,
								Fails:            0,
								RunFails:         0,
								RouteDrops:       0,
								Ping: service.LoadBalancePing{
									Gateway: "ping.ubnt.com",
									Status:  "REACHABLE",
								},
								LastRouteDrop:    nil,
								LastRouteRecover: nil,
							},
							{
								Interface:        "eth3",
								Status:           "OK",
								FailoverOnlyMode: false,
								Pings:            1000,
								Fails:            0,
								RunFails:         0,
								RouteDrops:       0,
								Ping: service.LoadBalancePing{
									Gateway: "ping.ubnt.com",
									Status:  "REACHABLE",
								},
								LastRouteDrop:    parseTime("2006-01-02T15:04:05Z"),
								LastRouteRecover: nil,
							},
						},
					},
				}))
				Expect(err).NotTo(HaveOccurred())
			})
		})

	})
})

func parseTime(value string) *time.Time {
	t, err := time.Parse(time.RFC3339, value)
	if err != nil {
		GinkgoT().Fatal(err)
	}
	return &t
}
