package main

import (
	"github.com/caarlos0/env"
	"github.com/joho/godotenv"
	"github.com/rs/zerolog/log"

	_ "github.com/mattn/go-sqlite3"
)

type Environment struct {
	DiscordToken string `env:"DISCORD_TOKEN"`
	Port         uint   `env:"PORT" envDefault:"8080"`
}

func main() {
	err := godotenv.Load()
	if err != nil {
		log.Fatal().Err(err).Msg("error loading .env file")
	}

	environment := Environment{}
	if err := env.Parse(&environment); err != nil {
		log.Fatal().Err(err).Msg("failed to parse environment variables")
	}

	err = InitDB("test.db")
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
