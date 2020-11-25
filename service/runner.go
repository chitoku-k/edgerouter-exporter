package service

import (
	"time"
)

type Runner interface {
	LoadBalanceWatchdog() ([]LoadBalanceGroup, error)
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
