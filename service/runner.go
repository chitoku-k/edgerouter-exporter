package service

import (
	"context"
	"time"
)

type Runner interface {
	Version(ctx context.Context) (Version, error)
	DdnsStatus(ctx context.Context) ([]DdnsStatus, error)
	LoadBalanceWatchdog(ctx context.Context) ([]LoadBalanceGroup, error)
	PPPoEClientSessions(ctx context.Context) ([]PPPoEClientSession, error)
}

type Version struct {
	Version        string
	BuildID        string
	BuildOn        *time.Time
	Copyright      string
	HWModel        string
	HWSerialNumber string
	Uptime         string
}

type DdnsStatus struct {
	Interface    string
	IPAddress    string
	HostName     string
	LastUpdate   *time.Time
	UpdateStatus string
}

type LoadBalanceGroup struct {
	Name     string
	Statuses []LoadBalanceStatus
}

type LoadBalanceStatus struct {
	Interface        string
	Status           string
	FailoverOnlyMode bool
	Pings            int
	Fails            int
	RunFails         int
	RouteDrops       int
	Ping             LoadBalancePing
	LastRouteDrop    *time.Time
	LastRouteRecover *time.Time
}

type LoadBalancePing struct {
	Gateway string
	Status  string
}

type PPPoEClientSession struct {
	User            string
	Time            *time.Duration
	Protocol        string
	Interface       string
	RemoteIP        string
	TransmitPackets int64
	TransmitBytes   int64
	ReceivePackets  int64
	ReceiveBytes    int64
}
