package main

import (
	"context"
	"os/signal"
	"syscall"

	"github.com/chitoku-k/edgerouter-exporter/application/server"
	"github.com/chitoku-k/edgerouter-exporter/infrastructure/cmd"
	"github.com/chitoku-k/edgerouter-exporter/infrastructure/config"
	"github.com/sirupsen/logrus"
)

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	env, err := config.Get()
	if err != nil {
		logrus.Fatalf("Failed to initialize config: %v", err)
	}

	engine := server.NewEngine(
		env.Port,
		cmd.NewRunnerService(env.OpCommand, cmd.NewParser()),
	)
	err = engine.Start(ctx)
	if err != nil {
		logrus.Fatalf("Failed to start web server: %v", err)
	}
}
