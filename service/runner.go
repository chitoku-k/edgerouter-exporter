package service

import (
	"context"
	"net"
	"time"
)

type IPProtocol int

const (
	_ IPProtocol = iota
	IPv4
	IPv6
)

type Runner interface {
	Version(ctx context.Context) (Version, error)
	BGPStatus(ctx context.Context, protocol IPProtocol) (BGPStatus, error)
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

type BGPStatus struct {
	RouterID     string
	LocalAS      int
	TableVersion int
	ASPaths      int
	Communities  int
	Neighbors    []BGPNeighbor
}

type BGPNeighbor struct {
	Address          net.IP
	Version          int
	RemoteAS         int
	MessagesReceived int64
	MessagesSent     int64
	TableVersion     int
	InQueue          int64
	OutQueue         int64
	Uptime           *time.Duration
	State            string
	PrefixesReceived int64
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
