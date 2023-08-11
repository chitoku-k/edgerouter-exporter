use async_trait::async_trait;
use tokio::try_join;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::VtyshCommand,
    },
    service::{bgp::BGPStatusResult, Runner},
};

pub struct BGPRunner<E, P> {
    command: VtyshCommand,
    executor: E,
    parser: P,
}

impl<E, P> BGPRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Context<'static> = (), Item = BGPStatusResult> + Send + Sync,
{
    pub fn new(command: VtyshCommand, executor: E, parser: P) -> Self {
        Self {
            command,
            executor,
            parser,
        }
    }

    async fn ipv4(&self) -> anyhow::Result<BGPStatusResult> {
        let output = self.executor.output(&self.command, &["-c", "show ip bgp summary"]).await?;
        let result = self.parser.parse(&output, ())?.map(|mut status| {
            status.neighbors.retain(|n| n.neighbor.is_ipv4());
            status
        });
        Ok(result)
    }

    async fn ipv6(&self) -> anyhow::Result<BGPStatusResult> {
        let output = self.executor.output(&self.command, &["-c", "show bgp ipv6 summary"]).await?;
        let result = self.parser.parse(&output, ())?.map(|mut status| {
            status.neighbors.retain(|n| n.neighbor.is_ipv6());
            status
        });
        Ok(result)
    }
}

#[async_trait]
impl<E, P> Runner for BGPRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Context<'static> = (), Item = BGPStatusResult> + Send + Sync,
{
    type Item = (BGPStatusResult, BGPStatusResult);

    async fn run(&self) -> anyhow::Result<Self::Item> {
        try_join!(self.ipv4(), self.ipv6())
    }
}

#[cfg(test)]
mod tests {
    use std::{net::{IpAddr, Ipv4Addr, Ipv6Addr}, time::Duration};

    use indoc::indoc;
    use mockall::{mock, predicate::eq};
    use pretty_assertions::assert_eq;

    use crate::{
        domain::bgp::{BGPNeighbor, BGPStatus},
        infrastructure::cmd::runner::MockExecutor,
    };

    use super::*;

    mock! {
        BGPParser {}

        impl Parser for BGPParser {
            type Context<'a> = ();
            type Item = BGPStatusResult;

            fn parse(&self, input: &str, context: <Self as Parser>::Context<'static>) -> anyhow::Result<<Self as Parser>::Item>;
        }
    }

    #[tokio::test]
    async fn none() {
        let command = VtyshCommand::from("/opt/vyatta/sbin/ubnt_vtysh".to_string());

        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/opt/vyatta/sbin/ubnt_vtysh", &["-c", "show ip bgp summary"]))
            .returning(|_, _| Ok("".to_string()));
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/opt/vyatta/sbin/ubnt_vtysh", &["-c", "show bgp ipv6 summary"]))
            .returning(|_, _| Ok("".to_string()));

        let mut mock_parser = MockBGPParser::new();
        mock_parser
            .expect_parse()
            .times(2)
            .with(eq(""), eq(()))
            .returning(|_, _| Ok(None));

        let runner = BGPRunner::new(command, mock_executor, mock_parser);
        let actual = runner.run().await.unwrap();
        assert_eq!(actual, (None, None));
    }

    #[tokio::test]
    async fn ipv4_only() {
        let command = VtyshCommand::from("/opt/vyatta/sbin/ubnt_vtysh".to_string());
        let ipv4_output = indoc! {"
            BGP router identifier 192.0.2.1, local AS number 64496
            BGP table version is 128
            1 BGP AS-PATH entries
            2 BGP community entries

            Neighbor                 V   AS   MsgRcv    MsgSen TblVer   InQ   OutQ    Up/Down   State/PfxRcd
            192.0.2.2                4 64497 1000       5000     128      1      5  01:11:11               9
            192.0.2.3                4 64497 2000       6000     128      2      6  1d02h03m              10
            192.0.2.4                4 64497    0          0       0      0      0     never     Connect

            Total number of neighbors 3

            Total number of Established sessions 2
        "};

        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/opt/vyatta/sbin/ubnt_vtysh", &["-c", "show ip bgp summary"]))
            .returning(|_, _| Ok(ipv4_output.to_string()));
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/opt/vyatta/sbin/ubnt_vtysh", &["-c", "show bgp ipv6 summary"]))
            .returning(|_, _| Ok("".to_string()));

        let mut mock_parser = MockBGPParser::new();
        mock_parser
            .expect_parse()
            .times(1)
            .with(eq(ipv4_output), eq(()))
            .returning(|_, _| Ok(Some(BGPStatus {
                router_id: "192.0.2.1".to_string(),
                local_as: 64496,
                table_version: 128,
                as_paths: 1,
                communities: 2,
                ebgp_maximum_paths: Some(8),
                ibgp_maximum_paths: Some(4),
                neighbors: vec![
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 2)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 1000,
                        messages_sent: 5000,
                        table_version: 128,
                        in_queue: 1,
                        out_queue: 5,
                        uptime: Some(Duration::new(4271, 0)),
                        state: None,
                        prefixes_received: Some(9),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 3)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 2000,
                        messages_sent: 6000,
                        table_version: 128,
                        in_queue: 2,
                        out_queue: 6,
                        uptime: Some(Duration::new(93780, 0)),
                        state: None,
                        prefixes_received: Some(10),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 4)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 0,
                        messages_sent: 0,
                        table_version: 0,
                        in_queue: 0,
                        out_queue: 0,
                        uptime: None,
                        state: Some("Connect".to_string()),
                        prefixes_received: None,
                    },
                ],
                sessions: 2,
            })));
        mock_parser
            .expect_parse()
            .times(1)
            .with(eq(""), eq(()))
            .returning(|_, _| Ok(None));

        let runner = BGPRunner::new(command, mock_executor, mock_parser);
        let actual = runner.run().await.unwrap();
        assert_eq!(actual, (
            Some(BGPStatus {
                router_id: "192.0.2.1".to_string(),
                local_as: 64496,
                table_version: 128,
                as_paths: 1,
                communities: 2,
                ebgp_maximum_paths: Some(8),
                ibgp_maximum_paths: Some(4),
                neighbors: vec![
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 2)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 1000,
                        messages_sent: 5000,
                        table_version: 128,
                        in_queue: 1,
                        out_queue: 5,
                        uptime: Some(Duration::new(4271, 0)),
                        state: None,
                        prefixes_received: Some(9),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 3)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 2000,
                        messages_sent: 6000,
                        table_version: 128,
                        in_queue: 2,
                        out_queue: 6,
                        uptime: Some(Duration::new(93780, 0)),
                        state: None,
                        prefixes_received: Some(10),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 4)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 0,
                        messages_sent: 0,
                        table_version: 0,
                        in_queue: 0,
                        out_queue: 0,
                        uptime: None,
                        state: Some("Connect".to_string()),
                        prefixes_received: None,
                    },
                ],
                sessions: 2,
            }),
            None,
        ));
    }

    #[tokio::test]
    async fn ipv6_only() {
        let command = VtyshCommand::from("/opt/vyatta/sbin/ubnt_vtysh".to_string());
        let ipv6_output = indoc! {"
            BGP router identifier 192.0.2.1, local AS number 64496
            BGP table version is 128
            1 BGP AS-PATH entries
            2 BGP community entries

            Neighbor                 V   AS   MsgRcv    MsgSen TblVer   InQ   OutQ    Up/Down   State/PfxRcd
            2001:db8::2              4 64497 3000       7000     128      3      7  03:33:33              11
            2001:db8::3              4 64497 4000       8000     128      4      8  4d05h06m              12
            2001:db8::ffff:ffff:ffff:ffff4 64497    0          0       0      0      0     never     Connect


            Total number of neighbors 3

            Total number of Established sessions 2
        "};

        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/opt/vyatta/sbin/ubnt_vtysh", &["-c", "show ip bgp summary"]))
            .returning(|_, _| Ok("".to_string()));
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/opt/vyatta/sbin/ubnt_vtysh", &["-c", "show bgp ipv6 summary"]))
            .returning(|_, _| Ok(ipv6_output.to_string()));

        let mut mock_parser = MockBGPParser::new();
        mock_parser
            .expect_parse()
            .times(1)
            .with(eq(""), eq(()))
            .returning(|_, _| Ok(None));
        mock_parser
            .expect_parse()
            .times(1)
            .with(eq(ipv6_output), eq(()))
            .returning(|_, _| Ok(Some(BGPStatus {
                router_id: "192.0.2.1".to_string(),
                local_as: 64496,
                table_version: 128,
                as_paths: 1,
                communities: 2,
                ebgp_maximum_paths: Some(8),
                ibgp_maximum_paths: Some(4),
                neighbors: vec![
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x2)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 3000,
                        messages_sent: 7000,
                        table_version: 128,
                        in_queue: 3,
                        out_queue: 7,
                        uptime: Some(Duration::new(12813, 0)),
                        state: None,
                        prefixes_received: Some(11),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x3)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 4000,
                        messages_sent: 8000,
                        table_version: 128,
                        in_queue: 4,
                        out_queue: 8,
                        uptime: Some(Duration::new(363960, 0)),
                        state: None,
                        prefixes_received: Some(12),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0xffff, 0x0ffff, 0xffff, 0xffff)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 0,
                        messages_sent: 0,
                        table_version: 0,
                        in_queue: 0,
                        out_queue: 0,
                        uptime: None,
                        state: Some("Connect".to_string()),
                        prefixes_received: None,
                    },
                ],
                sessions: 2,
            })));

        let runner = BGPRunner::new(command, mock_executor, mock_parser);
        let actual = runner.run().await.unwrap();
        assert_eq!(actual, (
            None,
            Some(BGPStatus {
                router_id: "192.0.2.1".to_string(),
                local_as: 64496,
                table_version: 128,
                as_paths: 1,
                communities: 2,
                ebgp_maximum_paths: Some(8),
                ibgp_maximum_paths: Some(4),
                neighbors: vec![
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x2)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 3000,
                        messages_sent: 7000,
                        table_version: 128,
                        in_queue: 3,
                        out_queue: 7,
                        uptime: Some(Duration::new(12813, 0)),
                        state: None,
                        prefixes_received: Some(11),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x3)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 4000,
                        messages_sent: 8000,
                        table_version: 128,
                        in_queue: 4,
                        out_queue: 8,
                        uptime: Some(Duration::new(363960, 0)),
                        state: None,
                        prefixes_received: Some(12),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0xffff, 0x0ffff, 0xffff, 0xffff)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 0,
                        messages_sent: 0,
                        table_version: 0,
                        in_queue: 0,
                        out_queue: 0,
                        uptime: None,
                        state: Some("Connect".to_string()),
                        prefixes_received: None,
                    },
                ],
                sessions: 2,
            }),
        ));
    }

    #[tokio::test]
    async fn ipv4_and_ipv6() {
        let command = VtyshCommand::from("/opt/vyatta/sbin/ubnt_vtysh".to_string());
        let ipv4_output = indoc! {"
            BGP router identifier 192.0.2.1, local AS number 64496
            BGP table version is 128
            1 BGP AS-PATH entries
            2 BGP community entries

            Neighbor                 V   AS   MsgRcv    MsgSen TblVer   InQ   OutQ    Up/Down   State/PfxRcd
            192.0.2.2                4 64497 1000       5000     128      1      5  01:11:11               9
            192.0.2.3                4 64497 2000       6000     128      2      6  1d02h03m              10
            192.0.2.4                4 64497    0          0       0      0      0     never     Connect
            2001:db8::2              4 64497 3000       7000     128      3      7  03:33:33               0
            2001:db8::3              4 64497 4000       8000     128      4      8  4d05h06m               0
            2001:db8::ffff:ffff:ffff:ffff4 64497    0          0       0      0      0     never     Connect

            Total number of neighbors 6

            Total number of Established sessions 4
        "};
        let ipv6_output = indoc! {"
            BGP router identifier 192.0.2.1, local AS number 64496
            BGP table version is 128
            1 BGP AS-PATH entries
            2 BGP community entries

            Neighbor                 V   AS   MsgRcv    MsgSen TblVer   InQ   OutQ    Up/Down   State/PfxRcd
            2001:db8::2              4 64497 3000       7000     128      3      7  03:33:33              11
            2001:db8::3              4 64497 4000       8000     128      4      8  4d05h06m              12
            2001:db8::ffff:ffff:ffff:ffff4 64497    0          0       0      0      0     never     Connect

            Total number of neighbors 3

            Total number of Established sessions 2
        "};

        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/opt/vyatta/sbin/ubnt_vtysh", &["-c", "show ip bgp summary"]))
            .returning(|_, _| Ok(ipv4_output.to_string()));
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/opt/vyatta/sbin/ubnt_vtysh", &["-c", "show bgp ipv6 summary"]))
            .returning(|_, _| Ok(ipv6_output.to_string()));

        let mut mock_parser = MockBGPParser::new();
        mock_parser
            .expect_parse()
            .times(1)
            .with(eq(ipv4_output), eq(()))
            .returning(|_, _| Ok(Some(BGPStatus {
                router_id: "192.0.2.1".to_string(),
                local_as: 64496,
                table_version: 128,
                as_paths: 1,
                communities: 2,
                ebgp_maximum_paths: Some(8),
                ibgp_maximum_paths: Some(4),
                neighbors: vec![
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 2)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 1000,
                        messages_sent: 5000,
                        table_version: 128,
                        in_queue: 1,
                        out_queue: 5,
                        uptime: Some(Duration::new(4271, 0)),
                        state: None,
                        prefixes_received: Some(9),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 3)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 2000,
                        messages_sent: 6000,
                        table_version: 128,
                        in_queue: 2,
                        out_queue: 6,
                        uptime: Some(Duration::new(93780, 0)),
                        state: None,
                        prefixes_received: Some(10),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 4)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 0,
                        messages_sent: 0,
                        table_version: 0,
                        in_queue: 0,
                        out_queue: 0,
                        uptime: None,
                        state: Some("Connect".to_string()),
                        prefixes_received: None,
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x2)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 3000,
                        messages_sent: 7000,
                        table_version: 128,
                        in_queue: 3,
                        out_queue: 7,
                        uptime: Some(Duration::new(12813, 0)),
                        state: None,
                        prefixes_received: Some(0),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x3)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 4000,
                        messages_sent: 8000,
                        table_version: 128,
                        in_queue: 4,
                        out_queue: 8,
                        uptime: Some(Duration::new(363960, 0)),
                        state: None,
                        prefixes_received: Some(0),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0xffff, 0x0ffff, 0xffff, 0xffff)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 0,
                        messages_sent: 0,
                        table_version: 0,
                        in_queue: 0,
                        out_queue: 0,
                        uptime: None,
                        state: Some("Connect".to_string()),
                        prefixes_received: None,
                    },
                ],
                sessions: 4,
            })));
        mock_parser
            .expect_parse()
            .times(1)
            .with(eq(ipv6_output), eq(()))
            .returning(|_, _| Ok(Some(BGPStatus {
                router_id: "192.0.2.1".to_string(),
                local_as: 64496,
                table_version: 128,
                as_paths: 1,
                communities: 2,
                ebgp_maximum_paths: Some(8),
                ibgp_maximum_paths: Some(4),
                neighbors: vec![
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x2)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 3000,
                        messages_sent: 7000,
                        table_version: 128,
                        in_queue: 3,
                        out_queue: 7,
                        uptime: Some(Duration::new(12813, 0)),
                        state: None,
                        prefixes_received: Some(11),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x3)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 4000,
                        messages_sent: 8000,
                        table_version: 128,
                        in_queue: 4,
                        out_queue: 8,
                        uptime: Some(Duration::new(363960, 0)),
                        state: None,
                        prefixes_received: Some(12),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0xffff, 0x0ffff, 0xffff, 0xffff)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 0,
                        messages_sent: 0,
                        table_version: 0,
                        in_queue: 0,
                        out_queue: 0,
                        uptime: None,
                        state: Some("Connect".to_string()),
                        prefixes_received: None,
                    },
                ],
                sessions: 2,
            })));

        let runner = BGPRunner::new(command, mock_executor, mock_parser);
        let actual = runner.run().await.unwrap();
        assert_eq!(actual, (
            Some(BGPStatus {
                router_id: "192.0.2.1".to_string(),
                local_as: 64496,
                table_version: 128,
                as_paths: 1,
                communities: 2,
                ebgp_maximum_paths: Some(8),
                ibgp_maximum_paths: Some(4),
                neighbors: vec![
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 2)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 1000,
                        messages_sent: 5000,
                        table_version: 128,
                        in_queue: 1,
                        out_queue: 5,
                        uptime: Some(Duration::new(4271, 0)),
                        state: None,
                        prefixes_received: Some(9),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 3)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 2000,
                        messages_sent: 6000,
                        table_version: 128,
                        in_queue: 2,
                        out_queue: 6,
                        uptime: Some(Duration::new(93780, 0)),
                        state: None,
                        prefixes_received: Some(10),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 4)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 0,
                        messages_sent: 0,
                        table_version: 0,
                        in_queue: 0,
                        out_queue: 0,
                        uptime: None,
                        state: Some("Connect".to_string()),
                        prefixes_received: None,
                    },
                ],
                sessions: 4,
            }),
            Some(BGPStatus {
                router_id: "192.0.2.1".to_string(),
                local_as: 64496,
                table_version: 128,
                as_paths: 1,
                communities: 2,
                ebgp_maximum_paths: Some(8),
                ibgp_maximum_paths: Some(4),
                neighbors: vec![
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x2)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 3000,
                        messages_sent: 7000,
                        table_version: 128,
                        in_queue: 3,
                        out_queue: 7,
                        uptime: Some(Duration::new(12813, 0)),
                        state: None,
                        prefixes_received: Some(11),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x3)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 4000,
                        messages_sent: 8000,
                        table_version: 128,
                        in_queue: 4,
                        out_queue: 8,
                        uptime: Some(Duration::new(363960, 0)),
                        state: None,
                        prefixes_received: Some(12),
                    },
                    BGPNeighbor {
                        neighbor: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0xffff, 0x0ffff, 0xffff, 0xffff)),
                        version: 4,
                        remote_as: 64497,
                        messages_received: 0,
                        messages_sent: 0,
                        table_version: 0,
                        in_queue: 0,
                        out_queue: 0,
                        uptime: None,
                        state: Some("Connect".to_string()),
                        prefixes_received: None,
                    },
                ],
                sessions: 2,
            }),
        ));
    }
}
