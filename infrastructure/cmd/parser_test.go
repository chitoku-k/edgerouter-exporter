package cmd_test

import (
	"net"
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

	Describe("ParseBGPStatus()", func() {
		Context("when IPv4 address is requested", func() {
			Context("when empty data is given", func() {
				It("returns empty", func() {
					actual, err := parser.ParseBGPStatus(nil, service.IPv4)
					Expect(actual.Neighbors).To(BeEmpty())
					Expect(err).NotTo(HaveOccurred())
				})
			})

			Context("when neighbors are given", func() {
				It("returns IPv4 neighbors", func() {
					actual, err := parser.ParseBGPStatus([]string{
						"BGP router identifier 192.0.2.1, local AS number 64496",
						"BGP table version is 128",
						"1 BGP AS-PATH entries",
						"2 BGP community entries",
						"",
						"Neighbor                 V   AS   MsgRcv    MsgSen TblVer   InQ   OutQ    Up/Down   State/PfxRcd",
						"192.0.2.2                4 64497 1000       5000     128      1      5  01:11:11               9",
						"192.0.2.3                4 64497 2000       6000     128      2      6  02:22:22              10",
						"192.0.2.4                4 64497    0          0       0      0      0     never     Connect",
						"2001:db8::2              4 64497 3000       7000     128      3      7  03:33:33               0",
						"2001:db8::3              4 64497 4000       8000     128      4      8  04:44:44               0",
						"2001:db8::4              4 64497    0          0       0      0      0     never     Connect",
						"",
						"Total number of neighbors 6",
						"",
						"Total number of Established sessions 4",
					}, service.IPv4)
					Expect(actual).To(Equal(service.BGPStatus{
						RouterID:     "192.0.2.1",
						LocalAS:      64496,
						TableVersion: 128,
						ASPaths:      1,
						Communities:  2,
						Neighbors: []service.BGPNeighbor{
							{
								Address:          net.IPv4(192, 0, 2, 2),
								Version:          4,
								RemoteAS:         64497,
								MessagesReceived: 1000,
								MessagesSent:     5000,
								TableVersion:     128,
								InQueue:          1,
								OutQueue:         5,
								Uptime:           parseDurationUnit("1h11m11s"),
								PrefixesReceived: 9,
							},
							{
								Address:          net.IPv4(192, 0, 2, 3),
								Version:          4,
								RemoteAS:         64497,
								MessagesReceived: 2000,
								MessagesSent:     6000,
								TableVersion:     128,
								InQueue:          2,
								OutQueue:         6,
								Uptime:           parseDurationUnit("2h22m22s"),
								PrefixesReceived: 10,
							},
							{
								Address:          net.IPv4(192, 0, 2, 4),
								Version:          4,
								RemoteAS:         64497,
								MessagesReceived: 0,
								MessagesSent:     0,
								TableVersion:     0,
								InQueue:          0,
								OutQueue:         0,
								Uptime:           nil,
								State:            "Connect",
								PrefixesReceived: 0,
							},
						},
					}))
					Expect(err).NotTo(HaveOccurred())
				})
			})
		})

		Context("when IPv6 address is requested", func() {
			Context("when empty data is given", func() {
				It("returns empty", func() {
					actual, err := parser.ParseBGPStatus(nil, service.IPv6)
					Expect(actual.Neighbors).To(BeEmpty())
					Expect(err).NotTo(HaveOccurred())
				})
			})

			Context("when neighbors are given", func() {
				It("returns IPv6 neighbors", func() {
					actual, err := parser.ParseBGPStatus([]string{
						"BGP router identifier 192.0.2.1, local AS number 64496",
						"BGP table version is 128",
						"1 BGP AS-PATH entries",
						"2 BGP community entries",
						"",
						"Neighbor                 V   AS   MsgRcv    MsgSen TblVer   InQ   OutQ    Up/Down   State/PfxRcd",
						"192.0.2.2                4 64497 1000       5000     128      1      5  01:11:11               9",
						"192.0.2.3                4 64497 2000       6000     128      2      6  02:22:22              10",
						"192.0.2.4                4 64497    0          0       0      0      0     never     Connect",
						"2001:db8::2              4 64497 3000       7000     128      3      7  03:33:33              11",
						"2001:db8::3              4 64497 4000       8000     128      4      8  04:44:44              12",
						"2001:db8::4              4 64497    0          0       0      0      0     never     Connect",
						"",
						"Total number of neighbors 6",
						"",
						"Total number of Established sessions 4",
					}, service.IPv6)
					Expect(actual).To(Equal(service.BGPStatus{
						RouterID:     "192.0.2.1",
						LocalAS:      64496,
						TableVersion: 128,
						ASPaths:      1,
						Communities:  2,
						Neighbors: []service.BGPNeighbor{
							{
								Address:          net.IP{0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02},
								Version:          4,
								RemoteAS:         64497,
								MessagesReceived: 3000,
								MessagesSent:     7000,
								TableVersion:     128,
								InQueue:          3,
								OutQueue:         7,
								Uptime:           parseDurationUnit("3h33m33s"),
								PrefixesReceived: 11,
							},
							{
								Address:          net.IP{0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03},
								Version:          4,
								RemoteAS:         64497,
								MessagesReceived: 4000,
								MessagesSent:     8000,
								TableVersion:     128,
								InQueue:          4,
								OutQueue:         8,
								Uptime:           parseDurationUnit("4h44m44s"),
								PrefixesReceived: 12,
							},
							{
								Address:          net.IP{0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04},
								Version:          4,
								RemoteAS:         64497,
								MessagesReceived: 0,
								MessagesSent:     0,
								TableVersion:     0,
								InQueue:          0,
								OutQueue:         0,
								Uptime:           nil,
								State:            "Connect",
								PrefixesReceived: 0,
							},
						},
					}))
					Expect(err).NotTo(HaveOccurred())
				})
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
						Time:            parseDurationUnit("3723s"),
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
						Time:            parseDurationUnit("3723s"),
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
						Time:            parseDurationUnit("363960s"),
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

func parseDurationUnit(value string) *time.Duration {
	d, err := time.ParseDuration(value)
	if err != nil {
		GinkgoT().Fatal(err)
	}
	return &d
}
