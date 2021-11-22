package cmd

import (
	"context"
	"fmt"
	"os/exec"
	"strings"

	"github.com/chitoku-k/edgerouter-exporter/service"
)

type runnerService struct {
	OpCommand     string
	OpDdnsCommand string
	VtyshCommand  string
	Parser        Parser
}

func NewRunnerService(opCommand, opDdnsCommand, vtyshCommand string, parser Parser) service.Runner {
	return &runnerService{
		OpCommand:     opCommand,
		OpDdnsCommand: opDdnsCommand,
		VtyshCommand:  vtyshCommand,
		Parser:        parser,
	}
}

func (s *runnerService) Version(ctx context.Context) (service.Version, error) {
	result, err := exec.CommandContext(ctx, s.OpCommand, "show", "version").Output()
	if err != nil {
		return service.Version{}, fmt.Errorf("failed to exec version: %w", err)
	}

	return s.Parser.ParseVersion(strings.Split(string(result), "\n"))
}

func (s *runnerService) BGPStatus(ctx context.Context, protocol service.IPProtocol) (service.BGPStatus, error) {
	var cmd string
	switch protocol {
	case service.IPv4:
		cmd = "show ip bgp summary"

	case service.IPv6:
		cmd = "show bgp ipv6 summary"

	default:
		return service.BGPStatus{}, fmt.Errorf("invalid protocol")
	}

	result, err := exec.CommandContext(ctx, s.VtyshCommand, "-c", cmd).Output()
	if err != nil {
		return service.BGPStatus{}, fmt.Errorf("failed to exec %v: %w", cmd, err)
	}

	return s.Parser.ParseBGPStatus(strings.Split(string(result), "\n"), protocol)
}

func (s *runnerService) DdnsStatus(ctx context.Context) ([]service.DdnsStatus, error) {
	result, err := exec.CommandContext(ctx, s.OpDdnsCommand, "--show-status").Output()
	if err != nil {
		return nil, fmt.Errorf("failed to exec ddns status: %w", err)
	}

	return s.Parser.ParseDdnsStatus(strings.Split(string(result), "\n"))
}

func (s *runnerService) LoadBalanceWatchdog(ctx context.Context) ([]service.LoadBalanceGroup, error) {
	result, err := exec.CommandContext(ctx, s.OpCommand, "show", "load-balance", "watchdog").Output()
	if err != nil {
		return nil, fmt.Errorf("failed to exec load-balance watchdog: %w", err)
	}

	return s.Parser.ParseLoadBalanceWatchdog(strings.Split(string(result), "\n"))
}

func (s *runnerService) PPPoEClientSessions(ctx context.Context) ([]service.PPPoEClientSession, error) {
	result, err := exec.CommandContext(ctx, s.OpCommand, "show", "pppoe-client").Output()
	if err != nil {
		return nil, fmt.Errorf("failed to get pppoe-client sessions: %w", err)
	}

	return s.Parser.ParsePPPoEClientSessions(strings.Split(string(result), "\n"))
}
