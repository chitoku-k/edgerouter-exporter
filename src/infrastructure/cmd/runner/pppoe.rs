use async_trait::async_trait;

use crate::{
    infrastructure::{
        cmd::{parser::Parser, runner::Executor},
        config::env::OpCommand,
    },
    service::{pppoe::PPPoEClientSessionResult, Runner},
};

#[derive(Clone)]
pub struct PPPoERunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = PPPoEClientSessionResult> + Send + Sync,
{
    command: OpCommand,
    executor: E,
    parser: P,
}

impl<E, P> PPPoERunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = PPPoEClientSessionResult> + Send + Sync,
{
    pub fn new(command: &OpCommand, executor: E, parser: P) -> Self {
        let command = command.to_owned();
        Self {
            command,
            executor,
            parser,
        }
    }

    async fn sessions(&self) -> anyhow::Result<PPPoEClientSessionResult> {
        let output = self.executor.output(&self.command, &["show", "pppoe-client"]).await?;
        let result = self.parser.parse(&output)?;
        Ok(result)
    }
}

#[async_trait]
impl<E, P> Runner for PPPoERunner<E, P>
where
    E: Executor + Send + Sync,
    P: Parser<Item = PPPoEClientSessionResult> + Send + Sync,
{
    type Item = PPPoEClientSessionResult;

    async fn run(&self) -> anyhow::Result<Self::Item> {
        self.sessions().await
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr},
        time::Duration,
    };

    use cool_asserts::assert_matches;
    use indoc::indoc;
    use mockall::{mock, predicate::eq};
    use number_prefix::{NumberPrefix, Prefix};

    use crate::{
        domain::pppoe::PPPoEClientSession,
        infrastructure::cmd::runner::MockExecutor,
    };

    use super::*;

    mock! {
        PPPoEParser {}

        impl Parser for PPPoEParser {
            type Item = PPPoEClientSessionResult;

            fn parse(&self, input: &str) -> anyhow::Result<<Self as Parser>::Item>;
        }
    }

    #[tokio::test]
    async fn sessions() {
        let command = OpCommand::from("/opt/vyatta/bin/vyatta-op-cmd-wrapper".to_string());
        let output = indoc! {"
            Active PPPoE client sessions:

            User       Time      Proto Iface   Remote IP       TX pkt/byte   RX pkt/byte
            ---------- --------- ----- -----   --------------- ------ ------ ------ ------
            user01     01h02m03s PPPoE pppoe0  192.0.2.255   384  34.8K   1.2K  58.2K
            user02     04d05h06m PPPoE pppoe1  198.51.100.255   768  76.8K   2.4K 116.4K

            Total sessions: 2
        "};

        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_output()
            .times(1)
            .returning(|command, args| {
                match (command, args) {
                    ("/opt/vyatta/bin/vyatta-op-cmd-wrapper", &["show", "pppoe-client"]) => Ok(output.to_string()),
                    _ => panic!("unexpected args"),
                }
            });

        let mut mock_parser = MockPPPoEParser::new();
        mock_parser
            .expect_parse()
            .times(1)
            .with(eq(output))
            .returning(|_| Ok(vec![
                PPPoEClientSession {
                    user: "user01".to_string(),
                    time: Duration::new(3723, 0),
                    protocol: "PPPoE".to_string(),
                    interface: "pppoe0".to_string(),
                    remote_ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 255)),
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
                    transmit_packets: NumberPrefix::Standalone(768.0).into(),
                    transmit_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 76.8).into(),
                    receive_packets: NumberPrefix::Prefixed(Prefix::Kilo, 2.4).into(),
                    receive_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 116.4).into(),
                },
            ]));

        let runner = PPPoERunner::new(&command, mock_executor, mock_parser);
        assert_matches!(
            runner.run().await,
            Ok(sessions) if sessions == vec![
                PPPoEClientSession {
                    user: "user01".to_string(),
                    time: Duration::new(3723, 0),
                    protocol: "PPPoE".to_string(),
                    interface: "pppoe0".to_string(),
                    remote_ip: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 255)),
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
                    transmit_packets: NumberPrefix::Standalone(768.0).into(),
                    transmit_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 76.8).into(),
                    receive_packets: NumberPrefix::Prefixed(Prefix::Kilo, 2.4).into(),
                    receive_bytes: NumberPrefix::Prefixed(Prefix::Kilo, 116.4).into(),
                },
            ],
        );
    }
}
