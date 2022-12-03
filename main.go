package main

import (
	"os"

	"github.com/caarlos0/env"
	"github.com/joho/godotenv"
	"github.com/rs/zerolog/log"
)

type Environment struct {
	DatabaseURL  string `env:"DATABASE_URL" envDefault:"postgresql://127.0.0.1/heaven" envExpand:"true"`
	DiscordToken string `env:"DISCORD_TOKEN"`
	Port         uint   `env:"PORT" envDefault:"8080"`
}

func main() {
	if _, err := os.Stat(".env"); err == nil {
		if err := godotenv.Load(); err != nil {
			log.Fatal().Err(err).Msg("failed to load .env file")
		}
	}
	environment := Environment{}
	if err := env.Parse(&environment); err != nil {
		log.Fatal().Err(err).Msg("failed to parse environment variables")
	}

	err := InitDB(environment.DatabaseURL)
	if err != nil {
		panic(err)
	}

	err = InitBot(environment.DiscordToken)
	if err != nil {
		log.Fatal().Err(err).Msg("failed to start discord bot")
	}
	defer session.Close()

	err = StartServer(environment.Port)
	if err != nil {
		panic(err)
	}
}
