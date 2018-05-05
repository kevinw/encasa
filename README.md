# encasa

## a personal new tab page for getting s\*\*t done

I'm a big fan of the [Todo.txt](http://todotxt.org/) format--there are great vim plugins to edit Todo.txt files, as well as iOS apps for updating Todo.txt files on Dropbox. The only missing piece was a way to see all my various projects' TODO files in one place.

![screenshot](https://raw.githubusercontent.com/kevinw/encasa/master/docs/static/screenshot.jpg)

## Setup

Put a file called homepage.yaml in your $HOME directory:

```yaml
local:
  - path: ~/Dropbox/Todo.txt
    name: todo
    todos: true
  - path: ~/Dropbox/Podcast/todo.txt
    name: podcast todo
    todos: true
    frequency_goal_seconds: 1day
    auto_project: podcast
  - path: ~/Dropbox/Game Ideas.txt
    name: Game Ideas
    frequency_goal_seconds: 1day
  - path: ~/src/homepage/TODO.txt
    name: "homepage todo"
    todos: true
    auto_project: homepage
  - path: ~/Dropbox/Journal.txt
    name: journal
    frequency_goal_seconds: 2days
```

## Rust libraries used

* Gotham for the web api
* askama for type-safe simple HTML templating
* humantime for parsing human readable times
* and of course, serde for serializing


