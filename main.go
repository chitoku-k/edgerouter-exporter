package main

import (
	"context"
	"os/signal"

	"github.com/chitoku-k/edgerouter-exporter/application/server"
	"github.com/chitoku-k/edgerouter-exporter/infrastructure/cmd"
	"github.com/chitoku-k/edgerouter-exporter/infrastructure/config"
	"github.com/sirupsen/logrus"
	"golang.org/x/sys/unix"
)

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), unix.SIGINT, unix.SIGTERM)
	defer stop()

	env, err := config.Get()
	if err != nil {
		logrus.Fatalf("Failed to initialize config: %v", err)
	}

	engine := server.NewEngine(
		env.Port,
		env.TLSCert,
		env.TLSKey,
		cmd.NewRunnerService(env.OpCommand, env.OpDdnsCommand, env.VtyshCommand, cmd.NewParser()),
	)
	err = engine.Start(ctx)
	if err != nil {
		logrus.Fatalf("Failed to start web server: %v", err)
	}
}
