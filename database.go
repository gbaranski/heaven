package main

import (
	"database/sql"

	"github.com/google/uuid"
	_ "github.com/lib/pq"
)

type Server struct {
	ID      uuid.UUID
	Token   uuid.UUID
	Address string
	GuildID string
}

type Angel struct {
	Name          string `json:"name"`
	UserID        string `json:"user-id"`
	ServerID      string `json:"server-id"`
	MinecraftName string `json:"minecraft-name"`
}

const create string = `
  CREATE TABLE IF NOT EXISTS angels (
	name 	       TEXT NOT NULL,
	minecraft_name TEXT NOT NULL,
	server_id      TEXT NOT NULL,
	user_id        TEXT NOT NULL,

	UNIQUE(server_id, minecraft_name),
	PRIMARY KEY(server_id, user_id)
  );

  CREATE TABLE IF NOT EXISTS servers (
	id 	     TEXT NOT NULL PRIMARY KEY,
	token    TEXT NOT NULL,
	address  TEXT NOT NULL,
	guild_id TEXT NOT NULL,
	UNIQUE(address, guild_id)
  );
`

var db *sql.DB

func InitDB(url string) error {
	var err error
	db, err = sql.Open("postgres", url)
	if err != nil {
		return err
	}

	if _, err := db.Exec(create); err != nil {
		return err
	}
	return db.Ping()
}

func AddServer(server *Server) error {
	_, err := db.Exec("INSERT INTO servers VALUES ($1, $2, $3, $4)", server.ID, server.Token, server.Address, server.GuildID)
	return err
}

func GetServer(serverID string) (*Server, error) {
	row := db.QueryRow("SELECT id, token, address, guild_id FROM servers WHERE id = $1", serverID)
	server := &Server{}
	if err := row.Scan(&server.ID, &server.Token, &server.Address, &server.GuildID); err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		} else {
			return nil, err
		}
	}
	return server, nil
}

func GetServerByAddress(guildID string, address string) (*Server, error) {
	row := db.QueryRow("SELECT id, token, address, guild_id FROM servers WHERE guild_id = $1 AND address = $2", guildID, address)
	server := &Server{}
	if err := row.Scan(&server.ID, &server.Token, &server.Address, &server.GuildID); err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		} else {
			return nil, err
		}
	}
	return server, nil
}

func AddAngel(angel *Angel) error {
	_, err := db.Exec("INSERT INTO angels VALUES ($1, $2, $3, $4)", angel.Name, angel.MinecraftName, angel.ServerID, angel.UserID)
	return err
}

func GetAngel(serverID string, userID string) (*Angel, error) {
	row := db.QueryRow("SELECT name, minecraft_name, server_id, user_id FROM angels WHERE server_id = $1 AND user_id = $2", serverID, userID)
	angel := &Angel{}
	if err := row.Scan(&angel.Name, &angel.MinecraftName, &angel.ServerID, &angel.UserID); err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		} else {
			return nil, err
		}
	}
	return angel, nil
}

func GetAngelByMinecraftName(serverID string, minecraftName string) (*Angel, error) {
	row := db.QueryRow("SELECT name, minecraft_name, server_id, user_id FROM angels WHERE server_id = $1 AND minecraft_name = $2", serverID, minecraftName)
	angel := &Angel{}
	if err := row.Scan(&angel.Name, &angel.MinecraftName, &angel.ServerID, &angel.UserID); err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		} else {
			return nil, err
		}
	}
	return angel, nil
}

func UpdateAngel(angel *Angel) error {
	_, err := db.Exec("UPDATE angels SET name = $1, minecraft_name = $2 WHERE server_id = $3 AND user_id = $4", angel.Name, angel.MinecraftName, angel.ServerID, angel.UserID)
	return err
}

func DoesAngelExistWithUserID(serverID string, userID string) (bool, error) {
	row := db.QueryRow("SELECT 1 FROM angels WHERE server_id = $1 AND user_id = $2", serverID, userID)
	var v string
	if err := row.Scan(&v); err != nil {
		if err == sql.ErrNoRows {
			return false, nil
		} else {
			return false, err
		}
	}
	return true, nil
}

func DoesAngelExistWithMinecraftName(serverID string, minecraftName string) (bool, error) {
	row := db.QueryRow("SELECT 1 FROM angels WHERE server_id = $1 AND minecraft_name = $2", serverID, minecraftName)
	var v string
	if err := row.Scan(&v); err != nil {
		if err == sql.ErrNoRows {
			return false, nil
		} else {
			return false, err
		}
	}
	return true, nil
}
