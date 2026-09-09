# Soundlore

A fast local music library where you design your own tagging systems and every tag becomes a playlist you can mix.

## Tagging

In other music players tagging tracks with an elaborate personal tagging system is rather slow and requires clicking around for several things, it usually goes like this:

- You click the track to hear it so you can classify it more accurately.
- You start adding the track to playlists either by drag and dropping it around or by clicking through a context menu.

Which isn't much for a library of a hundred tracks, but it certainly becomes a huge undertaking for a music library that goes beyond the one thousand file count mark, to make tagging much more faster and quick Soundlore allows you to tag tracks using keyboard controls in a special UI made with big tagging systems in mind, just select the tracks you wish to tag in the table, right click and select the option for tagging track!

![Tagging Modal UI](images/tag-tracks-modal-ui.png)

### Keyboard Controls

- **Alphanumeric Characters (0-9,a-z):** toggles the tag that corresponds to the character, ordered by key order left to right in a qwerty keyboard layout.
- **Tab:** Selects the next tag group, loops to the start if the last tag group is currently selected.
- **Shift + Tab:** Selects the previous tag group, loops to the end if the first tag group is currently selected. 
- **Enter:** Goes to the next track in the queue.
- **Shift + Enter:** Goes to the previous track in the queue.
- **Ctrl + Enter:** Goes to the last track in the queue.
- **Ctrl + Shift + Enter:** Goes to the first track in the queue.
- **Space:** Pauses or resumes playback.
- **Right Arrow:** Fast forwards playback 5 seconds. 
- **Left Arrow:** Rewinds playback 5 seconds.

## Getting Started with Development

First, if you're on Linux install the build dependencies:

```sh
# Debian / Ubuntu
sudo apt-get install build-essential pkg-config libasound2-dev libdbus-1-dev

# Fedora / RHEL
sudo dnf install gcc pkgconf-pkg-config alsa-lib-devel dbus-devel

# Arch
sudo pacman -S --needed base-devel pkgconf alsa-lib dbus
```

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

## Additional Documentation

- [Architecture](./docs/architecture/README.md)
- [Known issues](./docs/known-issues.md)
- [Tech Debt](./docs/tech-debt.md)
- [Todos](./docs/todo.md)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
