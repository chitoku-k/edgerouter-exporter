package main

import (
	"context"
	"os"
	"os/signal"
	"syscall"

	"github.com/chitoku-k/edgerouter-exporter/application/server"
	"github.com/chitoku-k/edgerouter-exporter/infrastructure/cmd"
	"github.com/chitoku-k/edgerouter-exporter/infrastructure/config"
	"github.com/sirupsen/logrus"
)

func main() {
	ctx, cancel := context.WithCancel(context.Background())

	sig := make(chan os.Signal)
	signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
	go func() {
		defer close(sig)
		<-sig
		cancel()
	}()

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
