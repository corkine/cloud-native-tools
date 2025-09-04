package main

import (
	"log"

	_ "github.com/rclone/rclone/backend/all" // import all backends
	"github.com/rclone/rclone/cmd"
	_ "github.com/rclone/rclone/cmd/all"    // import all commands
	_ "github.com/rclone/rclone/lib/plugin" // import plugins
)

func main() {
	err := setupDynamicConfig()
	if err != nil {
		log.Fatalf("dynamic config setup failed: %v", err)
	}
	cmd.Main()
}
