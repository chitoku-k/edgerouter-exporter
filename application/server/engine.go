package server

import (
	"context"
	"fmt"
	"net"
	"net/http"

	"github.com/chitoku-k/edgerouter-exporter/service"
	"github.com/gin-gonic/gin"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"
)

type engine struct {
	Port   string
	Runner service.Runner
}

type Engine interface {
	Start(ctx context.Context) error
}

func NewEngine(
	port string,
	runner service.Runner,
) Engine {
	return &engine{
		Port:   port,
		Runner: runner,
	}
}

func (e *engine) Start(ctx context.Context) error {
	router := gin.New()
	router.Use(gin.Recovery())
	router.Use(gin.LoggerWithConfig(gin.LoggerConfig{
		Formatter: e.Formatter(),
		SkipPaths: []string{"/healthz"},
	}))

	router.Any("/healthz", func(c *gin.Context) {
		c.String(http.StatusOK, "OK")
	})

	router.GET("/metrics", func(c *gin.Context) {
		ddns := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "dynamic_dns_status",
			Help:      "Result of DDNS update",
		}, []string{"interface_name", "ip_address", "hostname"})

		health := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "load_balancer_health",
			Help:      "Result of watchdog",
		}, []string{"group_name", "interface_name"})

		pingHealth := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "load_balancer_ping_health",
			Help:      "Result of ping",
		}, []string{"group_name", "interface_name", "gateway"})

		pingTotal := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "load_balancer_ping_total",
			Help:      "Total number of pings",
		}, []string{"group_name", "interface_name"})

		pingFailTotal := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "load_balancer_ping_fail_total",
			Help:      "Total number of ping failures",
		}, []string{"group_name", "interface_name"})

		runFailTotal := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "load_balancer_run_fail_total",
			Help:      "Total number of run failures",
		}, []string{"group_name", "interface_name"})

		routeDropTotal := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "load_balancer_route_drop_total",
			Help:      "Total number of route drops",
		}, []string{"group_name", "interface_name"})

		ddnsStatuses, err := e.Runner.DdnsStatus()
		if err != nil {
			logrus.Errorf("Error in retrieving ddns: %v", err)
			c.Status(http.StatusInternalServerError)
			return
		}

		for _, status := range ddnsStatuses {
			label := prometheus.Labels{
				"interface_name": status.Interface,
				"ip_address":     status.IPAddress,
				"hostname":       status.HostName,
			}

			if status.UpdateStatus == "good" {
				ddns.With(label).Set(1)
			} else {
				ddns.With(label).Set(0)
			}
		}

		groups, err := e.Runner.LoadBalanceWatchdog()
		if err != nil {
			logrus.Errorf("Error in running watchdog: %v", err)
			c.Status(http.StatusInternalServerError)
			return
		}

		for _, group := range groups {
			for _, status := range group.Statuses {
				label := prometheus.Labels{
					"group_name":     group.Name,
					"interface_name": status.Interface,
				}
				pingLabel := prometheus.Labels{
					"group_name":     group.Name,
					"interface_name": status.Interface,
					"gateway":        status.Ping.Gateway,
				}

				if status.Status == "OK" || status.Status == "Running" {
					health.With(label).Set(1)
				} else {
					health.With(label).Set(0)
				}

				if status.Ping.Status == "REACHABLE" {
					pingHealth.With(pingLabel).Set(1)
				} else {
					pingHealth.With(pingLabel).Set(0)
				}

				pingTotal.With(label).Set(float64(status.Pings))
				pingFailTotal.With(label).Set(float64(status.Fails))
				runFailTotal.With(label).Set(float64(status.RunFails))
				routeDropTotal.With(label).Set(float64(status.RouteDrops))
			}
		}

		registry := prometheus.NewRegistry()
		registry.MustRegister(ddns, health, pingHealth, pingTotal, pingFailTotal, runFailTotal, routeDropTotal)

		handler := promhttp.HandlerFor(registry, promhttp.HandlerOpts{})
		handler.ServeHTTP(c.Writer, c.Request)
	})

	server := http.Server{
		Addr:    net.JoinHostPort("", e.Port),
		Handler: router,
	}

	var eg errgroup.Group
	eg.Go(func() error {
		<-ctx.Done()
		return server.Shutdown(context.Background())
	})

	err := server.ListenAndServe()
	if err == http.ErrServerClosed {
		return eg.Wait()
	}

	return err
}

func (e *engine) Formatter() gin.LogFormatter {
	return func(param gin.LogFormatterParams) string {
		remoteHost, _, err := net.SplitHostPort(param.Request.RemoteAddr)
		if remoteHost == "" || err != nil {
			remoteHost = "-"
		}

		bodySize := fmt.Sprintf("%v", param.BodySize)
		if param.BodySize == 0 {
			bodySize = "-"
		}

		referer := param.Request.Header.Get("Referer")
		if referer == "" {
			referer = "-"
		}

		userAgent := param.Request.Header.Get("User-Agent")
		if userAgent == "" {
			userAgent = "-"
		}

		forwardedFor := param.Request.Header.Get("X-Forwarded-For")
		if forwardedFor == "" {
			forwardedFor = "-"
		}

		return fmt.Sprintf(`%s %s %s [%s] "%s %s %s" %v %s "%s" "%s" "%s"%s`,
			remoteHost,
			"-",
			"-",
			param.TimeStamp.Format("02/Jan/2006:15:04:05 -0700"),
			param.Request.Method,
			param.Request.RequestURI,
			param.Request.Proto,
			param.StatusCode,
			bodySize,
			referer,
			userAgent,
			forwardedFor,
			"\n",
		)
	}
}
