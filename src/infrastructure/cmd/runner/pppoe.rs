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

#[derive(Clone)]
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
    PPPoEParser: Parser<Input = (String, Vec<Interface>), Item = PPPoEClientSessionResult> + Send + Sync,
    InterfaceParser: Parser<Input = String, Item = InterfaceResult> + Send + Sync,
{
    pub fn new(
        op_command: &OpCommand,
        ip_command: &IpCommand,
        executor: E,
        pppoe_parser: PPPoEParser,
        interface_parser: InterfaceParser,
    ) -> Self {
        let op_command = op_command.to_owned();
        let ip_command = ip_command.to_owned();
        Self {
            op_command,
            ip_command,
            executor,
            pppoe_parser,
            interface_parser,
        }
    }

    async fn sessions(&self, interfaces: Vec<Interface>) -> anyhow::Result<PPPoEClientSessionResult> {
        let output = self.executor.output(&self.op_command, &["show", "pppoe-client"]).await?;
        let result = self.pppoe_parser.parse((output, interfaces))?;
        Ok(result)
    }

    async fn interfaces(&self) -> anyhow::Result<InterfaceResult> {
        let output = self.executor.output(&self.ip_command, &["--json", "addr", "show"]).await?;
        let result = self.interface_parser.parse(output)?;
        Ok(result)
    }
}

#[async_trait]
impl<E, PPPoEParser, InterfaceParser> Runner for PPPoERunner<E, PPPoEParser, InterfaceParser>
where
    E: Executor + Send + Sync,
    PPPoEParser: Parser<Input = (String, Vec<Interface>), Item = PPPoEClientSessionResult> + Send + Sync,
    InterfaceParser: Parser<Input = String, Item = InterfaceResult> + Send + Sync,
{
    type Item = PPPoEClientSessionResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        let interfaces = self.interfaces().await?;
        self.sessions(interfaces).await
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
            type Input = (String, Vec<Interface>);
            type Item = PPPoEClientSessionResult;

            fn parse(&self, input: (String, Vec<Interface>)) -> anyhow::Result<<Self as Parser>::Item>;
        }
    }

    mock! {
        InterfaceParser {}

        impl Parser for InterfaceParser {
            type Input = String;
            type Item = InterfaceResult;

            fn parse(&self, input: String) -> anyhow::Result<<Self as Parser>::Item>;
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
            [{
                    "ifindex": 1,
                    "ifname": "lo",
                    "flags": ["LOOPBACK","UP","LOWER_UP"],
                    "mtu": 65536,
                    "qdisc": "noqueue",
                    "operstate": "UNKNOWN",
                    "group": "default",
                    "txqlen": 1000,
                    "link_type": "loopback",
                    "address": "00:00:00:00:00:00",
                    "broadcast": "00:00:00:00:00:00",
                    "addr_info": [{
                            "family": "inet",
                            "local": "127.0.0.1",
                            "prefixlen": 8,
                            "scope": "host",
                            "label": "lo",
                            "valid_life_time": 4294967295,
                            "preferred_life_time": 4294967295
                        },{
                            "family": "inet6",
                            "local": "::1",
                            "prefixlen": 128,
                            "scope": "host",
                            "valid_life_time": 4294967295,
                            "preferred_life_time": 4294967295
                        }]
                }
            ]
        "#};

        let interfaces = vec![
            Interface {
                ifindex: 1,
                ifname: "lo".to_string(),
                link: None,
                flags: vec![
                    "LOOPBACK".to_string(),
                    "UP".to_string(),
                    "LOWER_UP".to_string(),
                ],
                mtu: 65536,
                qdisc: "noqueue".to_string(),
                operstate: "UNKNOWN".to_string(),
                group: "default".to_string(),
                txqlen: 1000,
                link_type: "loopback".to_string(),
                address: Some("00:00:00:00:00:00".to_string()),
                link_pointtopoint: None,
                broadcast: Some("00:00:00:00:00:00".to_string()),
                addr_info: vec![
                    AddrInfo {
                        family: "inet".to_string(),
                        local: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                        address: None,
                        prefixlen: 8,
                        broadcast: None,
                        scope: "host".to_string(),
                        dynamic: None,
                        mngtmpaddr: None,
                        noprefixroute: None,
                        label: Some("lo".to_string()),
                        valid_life_time: 4294967295,
                        preferred_life_time: 4294967295,
                    },
                    AddrInfo {
                        family: "inet6".to_string(),
                        local: IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
                        address: None,
                        prefixlen: 128,
                        broadcast: None,
                        scope: "host".to_string(),
                        dynamic: None,
                        mngtmpaddr: None,
                        noprefixroute: None,
                        label: None,
                        valid_life_time: 4294967295,
                        preferred_life_time: 4294967295,
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
            .withf(|command, args| (command, args) == ("/bin/ip", &["--json", "addr", "show"]))
            .returning(|_, _| Ok(interface_output.to_string()));

        let mut mock_pppoe_parser = MockPPPoEParser::new();
        mock_pppoe_parser
            .expect_parse()
            .times(1)
            .with(eq((pppoe_output.to_string(), interfaces)))
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
            .with(eq(interface_output.to_string()))
            .returning(|_| Ok(vec![
                Interface {
                    ifindex: 1,
                    ifname: "lo".to_string(),
                    link: None,
                    flags: vec![
                        "LOOPBACK".to_string(),
                        "UP".to_string(),
                        "LOWER_UP".to_string(),
                    ],
                    mtu: 65536,
                    qdisc: "noqueue".to_string(),
                    operstate: "UNKNOWN".to_string(),
                    group: "default".to_string(),
                    txqlen: 1000,
                    link_type: "loopback".to_string(),
                    address: Some("00:00:00:00:00:00".to_string()),
                    link_pointtopoint: None,
                    broadcast: Some("00:00:00:00:00:00".to_string()),
                    addr_info: vec![
                        AddrInfo {
                            family: "inet".to_string(),
                            local: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                            address: None,
                            prefixlen: 8,
                            broadcast: None,
                            scope: "host".to_string(),
                            dynamic: None,
                            mngtmpaddr: None,
                            noprefixroute: None,
                            label: Some("lo".to_string()),
                            valid_life_time: 4294967295,
                            preferred_life_time: 4294967295,
                        },
                        AddrInfo {
                            family: "inet6".to_string(),
                            local: IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
                            address: None,
                            prefixlen: 128,
                            broadcast: None,
                            scope: "host".to_string(),
                            dynamic: None,
                            mngtmpaddr: None,
                            noprefixroute: None,
                            label: None,
                            valid_life_time: 4294967295,
                            preferred_life_time: 4294967295,
                        },
                    ],
                },
            ]));

        let runner = PPPoERunner::new(&op_command, &ip_command, mock_executor, mock_pppoe_parser, mock_interface_parser);
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
