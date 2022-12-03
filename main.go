package main

import (
	"os"
	"path"

	"github.com/caarlos0/env"
	"github.com/joho/godotenv"
	"github.com/rs/zerolog/log"

	_ "github.com/mattn/go-sqlite3"
)

type Environment struct {
	DatabasePath string `env:"DATABASE_PATH" envDefault:"$HOME/.local/share/heaven/database.db" envExpand:"true"`
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

	if err := os.MkdirAll(path.Dir(environment.DatabasePath), 0755); err != nil {
		log.Fatal().Err(err).Msg("failed to create database directory")
	}

	err = InitDB(environment.DatabasePath)
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
