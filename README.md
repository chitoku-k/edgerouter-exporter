edgerouter-exporter
===================

[![][workflow-badge]][workflow-link]

A Prometheus exporter for EdgeRouter DDNS and Load Balancers

## Requirements

- Go
- EdgeRouter

## Installation

```sh
$ CGO_ENABLED=0 GOOS=linux GOARCH=mipsle go build
```

```sh
# Port number (required)
export PORT=8080

# Op command (optional)
export OP_COMMAND=/opt/vyatta/bin/vyatta-op-cmd-wrapper
```

## Usage

```sh
$ ./edgerouter-exporter
```

## Prometheus Configuration

Example result:

```
# HELP edgerouter_dynamic_dns_status Result of DDNS update
# TYPE edgerouter_dynamic_dns_status gauge
edgerouter_dynamic_dns_status{hostname="1.example.com",interface_name="eth0",ip_address="192.0.2.1"} 1
edgerouter_dynamic_dns_status{hostname="2.example.com",interface_name="eth1",ip_address="192.0.2.2"} 0
# HELP edgerouter_load_balancer_health Result of watchdog
# TYPE edgerouter_load_balancer_health gauge
edgerouter_load_balancer_health{group_name="WAN_FAILOVER",interface_name="eth0"} 1
edgerouter_load_balancer_health{group_name="WAN_FAILOVER",interface_name="eth1"} 0
# HELP edgerouter_load_balancer_ping_fail_total Total number of ping failures
# TYPE edgerouter_load_balancer_ping_fail_total gauge
edgerouter_load_balancer_ping_fail_total{group_name="WAN_FAILOVER",interface_name="eth0"} 125
edgerouter_load_balancer_ping_fail_total{group_name="WAN_FAILOVER",interface_name="eth1"} 2
# HELP edgerouter_load_balancer_ping_health Result of ping
# TYPE edgerouter_load_balancer_ping_health gauge
edgerouter_load_balancer_ping_health{gateway="8.8.8.8",group_name="WAN_FAILOVER",interface_name="eth0"} 1
edgerouter_load_balancer_ping_health{gateway="8.8.8.8",group_name="WAN_FAILOVER",interface_name="eth1"} 0
# HELP edgerouter_load_balancer_ping_total Total number of pings
# TYPE edgerouter_load_balancer_ping_total gauge
edgerouter_load_balancer_ping_total{group_name="WAN_FAILOVER",interface_name="eth0"} 1000
edgerouter_load_balancer_ping_total{group_name="WAN_FAILOVER",interface_name="eth1"} 1000
# HELP edgerouter_load_balancer_route_drop_total Total number of route drops
# TYPE edgerouter_load_balancer_route_drop_total gauge
edgerouter_load_balancer_route_drop_total{group_name="WAN_FAILOVER",interface_name="eth0"} 5
edgerouter_load_balancer_route_drop_total{group_name="WAN_FAILOVER",interface_name="eth1"} 0
# HELP edgerouter_load_balancer_run_fail_total Total number of run failures
# TYPE edgerouter_load_balancer_run_fail_total gauge
edgerouter_load_balancer_run_fail_total{group_name="WAN_FAILOVER",interface_name="eth0"} 0
edgerouter_load_balancer_run_fail_total{group_name="WAN_FAILOVER",interface_name="eth1"} 0
```

### Spec

| Status | Condition                           |
|--------|-------------------------------------|
| 200    | Success.                            |
| 500    | Unexpected error calling a command. |

[workflow-link]:    https://github.com/chitoku-k/edgerouter-exporter/actions?query=branch:master
[workflow-badge]:   https://img.shields.io/github/workflow/status/chitoku-k/edgerouter-exporter/CI%20Workflow/master.svg?style=flat-square
