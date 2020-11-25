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

func (s *runnerService) LoadBalanceWatchdog() ([]service.LoadBalanceGroup, error) {
	result, err := exec.Command(s.Command, "show", "load-balance", "watchdog").CombinedOutput()
	if err != nil {
		return nil, fmt.Errorf("failed to exec load-balance watchdog")
	}

	return s.Parser.ParseLoadBalancerWatchdog(strings.Split(string(result), "\n"))
}
