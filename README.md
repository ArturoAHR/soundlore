# Soundlore

A fast local music library where you design your own tagging systems and every tag becomes a playlist you can mix.

## Getting Started with Development

To setup the tooling that this project use we use [mise](https://github.com/jdx/mise).

You will need to mark the repository as trusted with `mise trust`, always check `mise.toml` to verify what's going to be installed in your machine, then run:

```sh
mise install 
just setup
```

### Database Scripts

If you want to run any `sqlx` commands you'll need to set up your `.env` file with the `DATABASE_URL` environment variable, this will vary depending on your OS and **is not required to build or run the project normally**:

```sh
# Linux
DATABASE_URL="sqlite:/home/<USER>/.local/share/soundlore-dev/data.db"

# MacOS
DATABASE_URL="sqlite:/Users/<USER>/Library/Application Support/soundlore-dev/data.db"

# Windows
DATABASE_URL="sqlite:C:/Users/<USER>/AppData/Roaming/soundlore-dev/data.db"
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
