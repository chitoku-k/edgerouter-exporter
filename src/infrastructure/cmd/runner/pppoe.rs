use async_trait::async_trait;

use crate::{
    domain::interface::Interface,
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::{IpCommand, OpCommand},
    },
    service::{
        interface::InterfaceResult,
        pppoe::PPPoEClientSessionResult,
        Runner,
    },
};

pub struct PPPoERunner<E, PPPoEParser, InterfaceParser> {
    op_command: OpCommand,
    ip_command: IpCommand,
    executor: E,
    pppoe_parser: PPPoEParser,
    interface_parser: InterfaceParser,
}

impl<E, PPPoEParser, InterfaceParser> PPPoERunner<E, PPPoEParser, InterfaceParser>
where
    E: Executor + Send + Sync,
    for<'a> PPPoEParser: Parser<Input<'a> = (&'a str, &'a [Interface]), Item = PPPoEClientSessionResult> + Send + Sync,
    for<'a> InterfaceParser: Parser<Input<'a> = &'a str, Item = InterfaceResult> + Send + Sync,
{
    pub fn new(
        op_command: OpCommand,
        ip_command: IpCommand,
        executor: E,
        pppoe_parser: PPPoEParser,
        interface_parser: InterfaceParser,
    ) -> Self {
        Self {
            op_command,
            ip_command,
            executor,
            pppoe_parser,
            interface_parser,
        }
    }

    async fn sessions(&self, interfaces: &[Interface]) -> anyhow::Result<PPPoEClientSessionResult> {
        let output = self.executor.output(&self.op_command, &["show", "pppoe-client"]).await?;
        let result = self.pppoe_parser.parse((&output, interfaces))?;
        Ok(result)
    }

    async fn interfaces(&self) -> anyhow::Result<InterfaceResult> {
        let output = self.executor.output(&self.ip_command, &["--brief", "addr", "show"]).await?;
        let result = self.interface_parser.parse(&output)?;
        Ok(result)
    }
}

#[async_trait]
impl<E, PPPoEParser, InterfaceParser> Runner for PPPoERunner<E, PPPoEParser, InterfaceParser>
where
    E: Executor + Send + Sync,
    for<'a> PPPoEParser: Parser<Input<'a> = (&'a str, &'a [Interface]), Item = PPPoEClientSessionResult> + Send + Sync,
    for<'a> InterfaceParser: Parser<Input<'a> = &'a str, Item = InterfaceResult> + Send + Sync,
{
    type Item = PPPoEClientSessionResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        let interfaces = self.interfaces().await?;
        self.sessions(&interfaces).await
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr, Ipv6Addr},
        time::Duration,
    };

    use indoc::indoc;
    use mockall::{mock, predicate::eq};
    use number_prefix::{NumberPrefix, Prefix};
    use pretty_assertions::assert_eq;

    use crate::{
        domain::{
            interface::{AddrInfo, Interface},
            pppoe::PPPoEClientSession,
        },
        infrastructure::cmd::runner::MockExecutor,
    };

    use super::*;

    mock! {
        PPPoEParser {}

        impl Parser for PPPoEParser {
            type Input<'a> = (&'a str, &'a [Interface]);
            type Item = PPPoEClientSessionResult;

            fn parse<'a>(&self, input: (&'a str, &'a [Interface])) -> anyhow::Result<<Self as Parser>::Item>;
        }
    }

    mock! {
        InterfaceParser {}

        impl Parser for InterfaceParser {
            type Input<'a> = &'a str;
            type Item = InterfaceResult;

            fn parse(&self, input: &str) -> anyhow::Result<<Self as Parser>::Item>;
        }
    }

    #[tokio::test]
    async fn sessions() {
        let op_command = OpCommand::from("/opt/vyatta/bin/vyatta-op-cmd-wrapper".to_string());
        let ip_command = IpCommand::from("/bin/ip".to_string());
        let pppoe_output = indoc! {"
            Active PPPoE client sessions:

            User       Time      Proto Iface   Remote IP       TX pkt/byte   RX pkt/byte
            ---------- --------- ----- -----   --------------- ------ ------ ------ ------
            user01     01h02m03s PPPoE pppoe0  192.0.2.255   384  34.8K   1.2K  58.2K
            user02     04d05h06m PPPoE pppoe1  198.51.100.255   768  76.8K   2.4K 116.4K

            Total sessions: 2
        "};
        let interface_output = indoc! {r#"
            lo               UNKNOWN        127.0.0.1/8 ::1/128 
            imq0             DOWN           
            pppoe0           UP             203.0.113.1 peer 192.0.2.255/32 
        "#};

        let interfaces = [
            Interface {
                ifname: "lo".to_string(),
                operstate: "UNKNOWN".to_string(),
                addr_info: vec![
                    AddrInfo {
                        local: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                        address: None,
                        prefixlen: 8,
                    },
                    AddrInfo {
                        local: IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
                        address: None,
                        prefixlen: 128,
                    },
                ],
            },
            Interface {
                ifname: "imq0".to_string(),
                operstate: "DOWN".to_string(),
                addr_info: vec![],
            },
            Interface {
                ifname: "pppoe0".to_string(),
                operstate: "UP".to_string(),
                addr_info: vec![
                    AddrInfo {
                        local: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)),
                        address: Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 255))),
                        prefixlen: 32,
                    },
                ],
            },
        ];

        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/opt/vyatta/bin/vyatta-op-cmd-wrapper", &["show", "pppoe-client"]))
            .returning(|_, _| Ok(pppoe_output.to_string()));
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/bin/ip", &["--brief", "addr", "show"]))
            .returning(|_, _| Ok(interface_output.to_string()));

        let mut mock_pppoe_parser = MockPPPoEParser::new();
        mock_pppoe_parser
            .expect_parse()
            .times(1)
            .withf(move |output| output == &(pppoe_output, &interfaces[..]))
            .returning(|_| Ok(vec![
                PPPoEClientSession {
                    user: "user01".to_string(),
                    time: Duration::new(3723, 0),
                    protocol: "PPPoE".to_string(),
                    interface: "pppoe0".to_string(),
                    remote_ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 255)),
                    local_ip: None,
                    transmit_packets: NumberPrefix::Standalone(384.0).into(),
                    transmit_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 34.8).into(),
                    receive_packets: NumberPrefix::Prefixed(Prefix::Kilo, 1.2).into(),
                    receive_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 58.2).into(),
                },
                PPPoEClientSession {
                    user: "user02".to_string(),
                    time: Duration::new(363960, 0),
                    protocol: "PPPoE".to_string(),
                    interface: "pppoe1".to_string(),
                    remote_ip: IpAddr::V4(Ipv4Addr::new(198, 51, 100, 255)),
                    local_ip: None,
                    transmit_packets: NumberPrefix::Standalone(768.0).into(),
                    transmit_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 76.8).into(),
                    receive_packets: NumberPrefix::Prefixed(Prefix::Kilo, 2.4).into(),
                    receive_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 116.4).into(),
                },
            ]));

        let mut mock_interface_parser = MockInterfaceParser::new();
        mock_interface_parser
            .expect_parse()
            .times(1)
            .with(eq(interface_output))
            .returning(|_| Ok(vec![
                Interface {
                    ifname: "lo".to_string(),
                    operstate: "UNKNOWN".to_string(),
                    addr_info: vec![
                        AddrInfo {
                            local: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                            address: None,
                            prefixlen: 8,
                        },
                        AddrInfo {
                            local: IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
                            address: None,
                            prefixlen: 128,
                        },
                    ],
                },
                Interface {
                    ifname: "imq0".to_string(),
                    operstate: "DOWN".to_string(),
                    addr_info: vec![],
                },
                Interface {
                    ifname: "pppoe0".to_string(),
                    operstate: "UP".to_string(),
                    addr_info: vec![
                        AddrInfo {
                            local: IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)),
                            address: Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 255))),
                            prefixlen: 32,
                        },
                    ],
                },
            ]));

        let runner = PPPoERunner::new(op_command, ip_command, mock_executor, mock_pppoe_parser, mock_interface_parser);
        let actual = runner.run().await.unwrap();
        assert_eq!(actual, vec![
            PPPoEClientSession {
                user: "user01".to_string(),
                time: Duration::new(3723, 0),
                protocol: "PPPoE".to_string(),
                interface: "pppoe0".to_string(),
                remote_ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 255)),
                local_ip: None,
                transmit_packets: NumberPrefix::Standalone(384.0).into(),
                transmit_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 34.8).into(),
                receive_packets: NumberPrefix::Prefixed(Prefix::Kilo, 1.2).into(),
                receive_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 58.2).into(),
            },
            PPPoEClientSession {
                user: "user02".to_string(),
                time: Duration::new(363960, 0),
                protocol: "PPPoE".to_string(),
                interface: "pppoe1".to_string(),
                remote_ip: IpAddr::V4(Ipv4Addr::new(198, 51, 100, 255)),
                local_ip: None,
                transmit_packets: NumberPrefix::Standalone(768.0).into(),
                transmit_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 76.8).into(),
                receive_packets: NumberPrefix::Prefixed(Prefix::Kilo, 2.4).into(),
                receive_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 116.4).into(),
            },
        ]);
    }
}
