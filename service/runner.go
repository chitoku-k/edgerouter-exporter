package service

import (
	"time"
)

type Runner interface {
	DdnsStatus() ([]DdnsStatus, error)
	LoadBalanceWatchdog() ([]LoadBalanceGroup, error)
	PPPoEClientSessions() ([]PPPoEClientSession, error)
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
