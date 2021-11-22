package config

import (
	"fmt"
	"os"
	"strings"
)

type Environment struct {
	Port          string
	TLSCert       string
	TLSKey        string
	OpCommand     string
	OpDdnsCommand string
	VtyshCommand  string
}

func Get() (Environment, error) {
	var missing []string
	var env Environment

	for k, v := range map[string]*string{
		"TLS_CERT":        &env.TLSCert,
		"TLS_KEY":         &env.TLSKey,
		"OP_COMMAND":      &env.OpCommand,
		"OP_DDNS_COMMAND": &env.OpDdnsCommand,
		"VTYSH_COMMAND":   &env.VtyshCommand,
	} {
		*v = os.Getenv(k)
	}

	for k, v := range map[string]*string{
		"PORT": &env.Port,
	} {
		*v = os.Getenv(k)

		if *v == "" {
			missing = append(missing, k)
		}
	}

	if len(missing) > 0 {
		return env, fmt.Errorf("missing env(s): %s", strings.Join(missing, ", "))
	}

	if env.OpCommand == "" {
		env.OpCommand = "/opt/vyatta/bin/vyatta-op-cmd-wrapper"
	}

	if env.OpDdnsCommand == "" {
		env.OpDdnsCommand = "/opt/vyatta/bin/sudo-users/vyatta-op-dynamic-dns.pl"
	}

	if env.VtyshCommand == "" {
		env.VtyshCommand = "/opt/vyatta/sbin/ubnt_vtysh"
	}

	return env, nil
}
