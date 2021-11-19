package server

import (
	"context"
	"crypto/tls"
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
	Port     string
	CertFile string
	KeyFile  string
	Runner   service.Runner
}

type Engine interface {
	Start(ctx context.Context) error
}

func NewEngine(
	port string,
	certFile string,
	keyFile string,
	runner service.Runner,
) Engine {
	return &engine{
		Port:     port,
		CertFile: certFile,
		KeyFile:  keyFile,
		Runner:   runner,
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
		info := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "info",
			Help:      "Version info",
		}, []string{"version", "build_id", "model"})

		bgpMsgRcv := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "bgp_message_received_total",
			Help:      "Total number of BGP messages received",
		}, []string{"neighbor", "as", "table_version"})

		bgpMsgSen := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "bgp_message_sent_total",
			Help:      "Total number of BGP messages sent",
		}, []string{"neighbor", "as", "table_version"})

		bgpInQ := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "bgp_message_in_queue",
			Help:      "Number of BGP messages in incoming queue",
		}, []string{"neighbor", "as", "table_version"})

		bgpOutQ := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "bgp_message_out_queue",
			Help:      "Number of BGP messages in outgoing queue",
		}, []string{"neighbor", "as", "table_version"})

		bgpSessionSecondsTotal := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "bgp_session_seconds_total",
			Help:      "Total seconds for established BGP session",
		}, []string{"neighbor", "as", "table_version"})

		bgpPfxRcd := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "bgp_prefix_received_total",
			Help:      "Total number of BGP prefixes received",
		}, []string{"neighbor", "as", "table_version"})

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

		pppoeClientSessionSecondsTotal := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "pppoe_client_session_seconds_total",
			Help:      "Total seconds for PPPoE client session",
		}, []string{"user", "protocol", "interface_name", "ip_address"})

		pppoeClientSessionTransmitPacketsTotal := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "pppoe_client_session_transmit_packets_total",
			Help:      "Total transmit packets for PPPoE client session",
		}, []string{"user", "protocol", "interface_name", "ip_address"})

		pppoeClientSessionTransmitBytesTotal := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "pppoe_client_session_transmit_bytes_total",
			Help:      "Total transmit bytes for PPPoE client session",
		}, []string{"user", "protocol", "interface_name", "ip_address"})

		pppoeClientSessionReceivePacketsTotal := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "pppoe_client_session_receive_packets_total",
			Help:      "Total receive packets for PPPoE client session",
		}, []string{"user", "protocol", "interface_name", "ip_address"})

		pppoeClientSessionReceiveBytesTotal := prometheus.NewGaugeVec(prometheus.GaugeOpts{
			Namespace: "edgerouter",
			Name:      "pppoe_client_session_receive_bytes_total",
			Help:      "Total receive bytes for PPPoE client session",
		}, []string{"user", "protocol", "interface_name", "ip_address"})

		version, err := e.Runner.Version(c)
		if err != nil {
			logrus.Errorf("Error in retrieving version: %v", err)
			c.Status(http.StatusInternalServerError)
			return
		}

		info.With(prometheus.Labels{
			"version":  version.Version,
			"build_id": version.BuildID,
			"model":    version.HWModel,
		}).Set(1)

		bgpStatusIPv4, err := e.Runner.BGPStatus(c, service.IPv4)
		if err != nil {
			logrus.Errorf("Error in retrieving BGP status for IPv4: %v", err)
			c.Status(http.StatusInternalServerError)
			return
		}

		bgpStatusIPv6, err := e.Runner.BGPStatus(c, service.IPv6)
		if err != nil {
			logrus.Errorf("Error in retrieving BGP status for IPv6: %v", err)
			c.Status(http.StatusInternalServerError)
			return
		}

		for _, neighbor := range append(bgpStatusIPv4.Neighbors, bgpStatusIPv6.Neighbors...) {
			label := prometheus.Labels{
				"neighbor":      neighbor.Address.String(),
				"as":            fmt.Sprint(neighbor.RemoteAS),
				"table_version": fmt.Sprint(neighbor.TableVersion),
			}

			var uptime float64
			if neighbor.Uptime != nil {
				uptime = neighbor.Uptime.Seconds()
			}

			bgpMsgRcv.With(label).Set(float64(neighbor.MessagesReceived))
			bgpMsgSen.With(label).Set(float64(neighbor.MessagesSent))
			bgpInQ.With(label).Set(float64(neighbor.InQueue))
			bgpOutQ.With(label).Set(float64(neighbor.OutQueue))
			bgpSessionSecondsTotal.With(label).Set(uptime)
			bgpPfxRcd.With(label).Set(float64(neighbor.PrefixesReceived))
		}

		ddnsStatuses, err := e.Runner.DdnsStatus(c)
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

		groups, err := e.Runner.LoadBalanceWatchdog(c)
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

		pppoeClientSessions, err := e.Runner.PPPoEClientSessions(c)
		if err != nil {
			logrus.Errorf("Error in running PPPoE client sessions: %v", err)
			c.Status(http.StatusInternalServerError)
			return
		}

		for _, session := range pppoeClientSessions {
			label := prometheus.Labels{
				"user":           session.User,
				"protocol":       session.Protocol,
				"interface_name": session.Interface,
				"ip_address":     session.RemoteIP,
			}

			if session.Time != nil {
				pppoeClientSessionSecondsTotal.With(label).Set(session.Time.Seconds())
			}
			pppoeClientSessionTransmitPacketsTotal.With(label).Set(float64(session.TransmitPackets))
			pppoeClientSessionTransmitBytesTotal.With(label).Set(float64(session.TransmitBytes))
			pppoeClientSessionReceivePacketsTotal.With(label).Set(float64(session.ReceivePackets))
			pppoeClientSessionReceiveBytesTotal.With(label).Set(float64(session.ReceiveBytes))
		}

		registry := prometheus.NewRegistry()
		registry.MustRegister(
			info,
			bgpMsgRcv,
			bgpMsgSen,
			bgpInQ,
			bgpOutQ,
			bgpSessionSecondsTotal,
			bgpPfxRcd,
			ddns,
			health,
			pingHealth,
			pingTotal,
			pingFailTotal,
			runFailTotal,
			routeDropTotal,
			pppoeClientSessionSecondsTotal,
			pppoeClientSessionTransmitPacketsTotal,
			pppoeClientSessionTransmitBytesTotal,
			pppoeClientSessionReceivePacketsTotal,
			pppoeClientSessionReceiveBytesTotal,
		)

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

	var err error
	if e.CertFile != "" && e.KeyFile != "" {
		server.TLSConfig = &tls.Config{
			GetCertificate: e.getCertificate,
		}
		err = server.ListenAndServeTLS("", "")
	} else {
		err = server.ListenAndServe()
	}

	if err == http.ErrServerClosed {
		return eg.Wait()
	}

	return err
}

func (e *engine) getCertificate(*tls.ClientHelloInfo) (*tls.Certificate, error) {
	cert, err := tls.LoadX509KeyPair(e.CertFile, e.KeyFile)
	if err != nil {
		return nil, fmt.Errorf("failed to get certificate: %w", err)
	}

	return &cert, nil
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
