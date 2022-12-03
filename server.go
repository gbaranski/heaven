package main

import (
	"fmt"

	"github.com/gin-gonic/gin"
	"github.com/rs/zerolog/log"
)

func StartServer(port uint) error {
	r := gin.Default()
	r.POST("/:server-id/by-minecraft-name/:minecraft-name/authorize", func(c *gin.Context) {
		serverID := c.Param("server-id")
		server, err := GetServer(serverID)
		if err != nil {
			log.Error().Err(err).Msg("failed to get server")
			c.Status(500)
			return
		}
		if server == nil {
			c.Status(400)
			return
		}
		angel, err := GetAngelByMinecraftName(server.ID.String(), c.Param("minecraft-name"))
		if err != nil {
			log.Error().Err(err).Msg("failed to get angel")
			c.Status(500)
			return
		}
		if angel == nil {
			c.Status(404)
			return
		}

		from := c.Query("from")
		result, err := authorize(server, angel, from)
		if err != nil {
			log.Error().Err(err).Msg("failed to authorize")
			c.Status(500)
			return
		}
		if result {
			c.Status(200)
		} else {
			c.Status(401)
		}
	})

	r.GET("/:server-id/by-minecraft-name/:minecraft-name", func(c *gin.Context) {
		serverID := c.Param("server-id")
		server, err := GetServer(serverID)
		if err != nil {
			log.Error().Err(err).Msg("failed to get server")
			c.Status(500)
			return
		}
		if server == nil {
			c.Status(400)
			return
		}
		angel, err := GetAngelByMinecraftName(server.ID.String(), c.Param("minecraft-name"))
		if err != nil {
			log.Error().Err(err).Msg("failed to get angel")
			c.Status(500)
			return
		}
		if angel == nil {
			c.Status(404)
			return
		}

		c.JSON(200, angel)
	})

	return r.Run(":" + fmt.Sprint(port))
}
