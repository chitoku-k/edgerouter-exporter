use async_trait::async_trait;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::IpCommand,
    },
    service::{interface::InterfaceResult, Runner},
};

#[derive(Clone)]
pub struct InterfaceRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = InterfaceResult> + Send + Sync,
{
    command: IpCommand,
    executor: E,
    parser: P,
}

impl<E, P> InterfaceRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = InterfaceResult> + Send + Sync,
{
    pub fn new(command: &IpCommand, executor: E, parser: P) -> Self {
        let command = command.to_owned();
        Self {
            command,
            executor,
            parser,
        }
    }

    async fn interfaces(&self) -> anyhow::Result<InterfaceResult> {
        let output = self.executor.output(&self.command, &["--json", "addr", "show"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

#[async_trait]
impl<E, P> Runner for InterfaceRunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = InterfaceResult> + Send + Sync,
{
    type Item = InterfaceResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.interfaces().await
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use indoc::indoc;
    use mockall::{mock, predicate::eq};
    use pretty_assertions::assert_eq;

    use crate::{
        domain::interface::{AddrInfo, Interface},
        infrastructure::cmd::runner::MockExecutor,
    };

    use super::*;

    mock! {
        InterfaceParser {}

        impl Parser for InterfaceParser {
            type Item = InterfaceResult;

            fn parse(&self, input: &str) -> anyhow::Result<<Self as Parser>::Item>;
        }
    }

    #[tokio::test]
    async fn interfaces() {
        let command = IpCommand::from("/bin/ip".to_string());
        let output = indoc! {r#"
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

        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_output()
            .times(1)
            .withf(|command, args| (command, args) == ("/bin/ip", &["--json", "addr", "show"]))
            .returning(|_, _| Ok(output.to_string()));

        let mut mock_parser = MockInterfaceParser::new();
        mock_parser
            .expect_parse()
            .times(1)
            .with(eq(output))
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

        let runner = InterfaceRunner::new(&command, mock_executor, mock_parser);
        let actual = runner.run().await.unwrap();
        assert_eq!(actual, vec![
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
        ]);
    }
}
