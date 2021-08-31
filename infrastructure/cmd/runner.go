package cmd

import (
	"context"
	"fmt"
	"os/exec"
	"strings"

	"github.com/chitoku-k/edgerouter-exporter/service"
)

type runnerService struct {
	Command string
	Parser  Parser
}

func NewRunnerService(command string, parser Parser) service.Runner {
	return &runnerService{
		Command: command,
		Parser:  parser,
	}
}

func (s *runnerService) Version(ctx context.Context) (service.Version, error) {
	result, err := exec.CommandContext(ctx, s.Command, "show", "version").Output()
	if err != nil {
		return service.Version{}, fmt.Errorf("failed to exec version: %w", err)
	}

	return s.Parser.ParseVersion(strings.Split(string(result), "\n"))
}

func (s *runnerService) DdnsStatus(ctx context.Context) ([]service.DdnsStatus, error) {
	result, err := exec.CommandContext(ctx, s.Command, "show", "dns", "dynamic", "status").Output()
	if err != nil {
		return nil, fmt.Errorf("failed to exec ddns status: %w", err)
	}

	return s.Parser.ParseDdnsStatus(strings.Split(string(result), "\n"))
}

func (s *runnerService) LoadBalanceWatchdog(ctx context.Context) ([]service.LoadBalanceGroup, error) {
	result, err := exec.CommandContext(ctx, s.Command, "show", "load-balance", "watchdog").Output()
	if err != nil {
		return nil, fmt.Errorf("failed to exec load-balance watchdog: %w", err)
	}

	return s.Parser.ParseLoadBalanceWatchdog(strings.Split(string(result), "\n"))
}

func (s *runnerService) PPPoEClientSessions(ctx context.Context) ([]service.PPPoEClientSession, error) {
	result, err := exec.CommandContext(ctx, s.Command, "show", "pppoe-client").Output()
	if err != nil {
		return nil, fmt.Errorf("failed to get pppoe-client sessions: %w", err)
	}

	return s.Parser.ParsePPPoEClientSessions(strings.Split(string(result), "\n"))
}
