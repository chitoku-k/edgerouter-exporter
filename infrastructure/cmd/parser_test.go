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

	Describe("ParseVersion()", func() {
		Context("when emtpy data is given", func() {
			It("returns an error", func() {
				actual, err := parser.ParseVersion(nil)
				Expect(actual).To(Equal(service.Version{}))
				Expect(err).To(MatchError("expected version, found nothing"))
			})
		})

		Context("when version is given", func() {
			It("returns version", func() {
				actual, err := parser.ParseVersion([]string{
					"Version:      v2.0.6",
					"Build ID:     5208541",
					"Build on:     01/02/06 15:04",
					"Copyright:    2012-2018 Ubiquiti Networks, Inc.",
					"HW model:     EdgeRouter X 5-Port",
					"HW S/N:       000000000000",
					"Uptime:       01:00:00 up  1:00,  1 user,  load average: 1.00, 1.00, 1.00",
				})
				Expect(actual).To(Equal(service.Version{
					Version:        "v2.0.6",
					BuildID:        "5208541",
					BuildOn:        parseTime("2006-01-02T15:04:00Z"),
					Copyright:      "2012-2018 Ubiquiti Networks, Inc.",
					HWModel:        "EdgeRouter X 5-Port",
					HWSerialNumber: "000000000000",
					Uptime:         "01:00:00 up  1:00,  1 user,  load average: 1.00, 1.00, 1.00",
				}))
				Expect(err).NotTo(HaveOccurred())
			})
		})
	})

	Describe("ParseDdnsStatus()", func() {
		Context("when empty data is given", func() {
			It("returns empty", func() {
				actual, err := parser.ParseDdnsStatus(nil)
				Expect(actual).To(BeEmpty())
				Expect(err).NotTo(HaveOccurred())
			})
		})

		Context("when no status is given", func() {
			It("returns empty", func() {
				actual, err := parser.ParseDdnsStatus([]string{
					"Dynamic DNS not configured",
					"",
				})
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
					"last update  : Mon Jan  2 15:04:05 2006",
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
					"last update  : Mon Jan  2 15:04:05 2006",
					"update-status: good",
					"",
					"interface    : eth1 [ Currently no IP address ]",
					"host-name    : 2.example.com",
					"last update  : Mon Jan  2 15:04:06 2006",
					"update-status: ",
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
						HostName:     "2.example.com",
						LastUpdate:   parseTime("2006-01-02T15:04:06Z"),
						UpdateStatus: "",
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

		Context("when no group is given", func() {
			It("returns empty", func() {
				actual, err := parser.ParseDdnsStatus([]string{
					"load-balance is not configured",
				})
				Expect(actual).To(BeEmpty())
				Expect(err).NotTo(HaveOccurred())
			})
		})

		Context("when empty group is given", func() {
			It("returns empty", func() {
				actual, err := parser.ParseLoadBalanceWatchdog([]string{
					"Group FAILOVER_01",
				})
				Expect(actual).To(Equal([]service.LoadBalanceGroup{
					{
						Name: "FAILOVER_01",
					},
				}))
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
					"  last route drop   : Mon Jan  2 15:04:05 2006",
					"  last route recover: Mon Jan  2 15:04:00 2006",
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
					"  last route drop   : Mon Jan  2 15:04:05 2006",
					"  last route recover: Mon Jan  2 15:04:00 2006",
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
					"  last route drop   : Mon Jan  2 15:04:05 2006",
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

		Context("when invalid data is given", func() {
			It("returns an error", func() {
				actual, err := parser.ParseLoadBalanceWatchdog([]string{
					"mesg: ttyname failed: Inappropriate ioctl for device",
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
					"  last route drop   : Mon Jan  2 15:04:05 2006",
					"  last route recover: Mon Jan  2 15:04:00 2006",
					"",
				})
				Expect(actual).To(BeEmpty())
				Expect(err).To(MatchError("unexpected token, expecting group: mesg: ttyname failed: Inappropriate ioctl for device"))
			})
		})
	})

	Describe("ParsePPPoEClientSessions()", func() {
		Context("when empty data is given", func() {
			It("returns empty", func() {
				actual, err := parser.ParsePPPoEClientSessions(nil)
				Expect(actual).To(BeEmpty())
				Expect(err).NotTo(HaveOccurred())
			})
		})

		Context("when no session is given", func() {
			It("return empty", func() {
				actual, err := parser.ParsePPPoEClientSessions([]string{
					"No active PPPoE client sessions",
				})
				Expect(actual).To(BeEmpty())
				Expect(err).NotTo(HaveOccurred())
			})
		})

		Context("when one session is given", func() {
			It("returns one session", func() {
				actual, err := parser.ParsePPPoEClientSessions([]string{
					"Active PPPoE client sessions:",
					"",
					"User       Time      Proto Iface   Remote IP       TX pkt/byte   RX pkt/byte",
					"---------- --------- ----- -----   --------------- ------ ------ ------ ------",
					"user01     01h02m03s PPPoE pppoe0  192.0.2.255   384  34.8K   1.2K  58.2K",
					"",
					"Total sessions: 1",
				})
				Expect(actual).To(Equal([]service.PPPoEClientSession{
					{
						User:            "user01",
						Time:            parseDuration("3723s"),
						Protocol:        "PPPoE",
						Interface:       "pppoe0",
						RemoteIP:        "192.0.2.255",
						TransmitPackets: 384,
						TransmitBytes:   35635,
						ReceivePackets:  1228,
						ReceiveBytes:    59596,
					},
				}))
				Expect(err).NotTo(HaveOccurred())
			})
		})

		Context("when multiple sessions are given", func() {
			It("returns multiple sessions", func() {
				actual, err := parser.ParsePPPoEClientSessions([]string{
					"Active PPPoE client sessions:",
					"",
					"User       Time      Proto Iface   Remote IP       TX pkt/byte   RX pkt/byte",
					"---------- --------- ----- -----   --------------- ------ ------ ------ ------",
					"user01     01h02m03s PPPoE pppoe0  192.0.2.255   384  34.8K   1.2K  58.2K",
					"user02     04d05h06m PPPoE pppoe1  198.51.100.255   768  76.8K   2.4K 116.4K",
					"",
					"Total sessions: 1",
				})
				Expect(actual).To(Equal([]service.PPPoEClientSession{
					{
						User:            "user01",
						Time:            parseDuration("3723s"),
						Protocol:        "PPPoE",
						Interface:       "pppoe0",
						RemoteIP:        "192.0.2.255",
						TransmitPackets: 384,
						TransmitBytes:   35635,
						ReceivePackets:  1228,
						ReceiveBytes:    59596,
					},
					{
						User:            "user02",
						Time:            parseDuration("363960s"),
						Protocol:        "PPPoE",
						Interface:       "pppoe1",
						RemoteIP:        "198.51.100.255",
						TransmitPackets: 768,
						TransmitBytes:   78643,
						ReceivePackets:  2457,
						ReceiveBytes:    119193,
					},
				}))
				Expect(err).NotTo(HaveOccurred())
			})
		})

		Context("when invalid data is given", func() {
			It("returns an error", func() {
				actual, err := parser.ParsePPPoEClientSessions([]string{
					"Active PPPoE client sessions:",
					"",
					"Time      Proto Iface   Remote IP       TX pkt/byte   RX pkt/byte",
					"--------- ----- -----   --------------- ------ ------ ------ ------",
					"01h02m03s PPPoE pppoe0  192.0.2.255   384  34.8K   1.2K  58.2K",
					"",
					"Total sessions: 1",
				})
				Expect(actual).To(BeEmpty())
				Expect(err).To(MatchError("unexpected number of fields, expecting 9 fields: 01h02m03s PPPoE pppoe0  192.0.2.255   384  34.8K   1.2K  58.2K"))
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

func parseDuration(value string) *time.Duration {
	d, err := time.ParseDuration(value)
	if err != nil {
		GinkgoT().Fatal(err)
	}
	return &d
}
