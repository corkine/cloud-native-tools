package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
)

type RemoteConfig struct {
	Name       string                 `json:"name"`
	Parameters map[string]interface{} `json:"config"`
}

func setupDynamicConfig() error {
	var configURL string
	flag.StringVar(&configURL, "config_url", "", "URL to fetch remote configuration")

	flag.Parse()

	newArgs := []string{os.Args[0]}

	for i := 1; i < len(os.Args); i++ {
		arg := os.Args[i]

		if arg == "--config_url" {
			i++
			continue
		}

		newArgs = append(newArgs, arg)
	}

	os.Args = newArgs

	flag.CommandLine = flag.NewFlagSet(os.Args[0], flag.ExitOnError)

	if configURL == "" {
		configURL = os.Getenv("RCLONE_CONFIG_URL")
		if configURL == "" {
			return nil
		}
	}

	resp, err := http.Get(configURL)
	if err != nil {
		return fmt.Errorf("failed to get config: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("config server return error status: %s", resp.Status)
	}

	var config RemoteConfig
	if err := json.NewDecoder(resp.Body).Decode(&config); err != nil {
		return fmt.Errorf("failed to parse json config: %w", err)
	}

	log.Printf("using remote '%s' config", config.Name)

	remoteNameUpper := strings.ToUpper(config.Name)
	for key, value := range config.Parameters {
		envKey := fmt.Sprintf("RCLONE_CONFIG_%s_%s", remoteNameUpper, strings.ToUpper(key))

		var strValue string
		switch v := value.(type) {
		case string:
			strValue = v
		case int, int64, float64:
			strValue = fmt.Sprintf("%v", v)
		case bool:
			strValue = fmt.Sprintf("%t", v)
		default:
			if jsonBytes, err := json.Marshal(v); err == nil {
				strValue = string(jsonBytes)
			} else {
				strValue = fmt.Sprintf("%v", v)
			}
		}

		if err := os.Setenv(envKey, strValue); err != nil {
			return fmt.Errorf("failed to set environment variable %s: %w", envKey, err)
		}
	}

	return nil
}
