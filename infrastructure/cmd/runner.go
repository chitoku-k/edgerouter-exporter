package cmd

import (
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

func (s *runnerService) DdnsStatus() ([]service.DdnsStatus, error) {
	result, err := exec.Command(s.Command, "show", "dns", "dynamic", "status").Output()
	if err != nil {
		return nil, fmt.Errorf("failed to exec ddns status: %w", err)
	}

	return s.Parser.ParseDdnsStatus(strings.Split(string(result), "\n"))
}

func (s *runnerService) LoadBalanceWatchdog() ([]service.LoadBalanceGroup, error) {
	result, err := exec.Command(s.Command, "show", "load-balance", "watchdog").Output()
	if err != nil {
		return nil, fmt.Errorf("failed to exec load-balance watchdog: %w", err)
	}

	return s.Parser.ParseLoadBalanceWatchdog(strings.Split(string(result), "\n"))
}
