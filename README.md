# encasa

**An opinionated personal new tab page for tracking todo.txts and events**

![build status](https://travis-ci.org/kevinw/encasa.svg?branch=master)

I'm a big fan of the [Todo.txt](http://todotxt.org/) format--there are great vim plugins to edit Todo.txt files, as well as iOS apps for updating Todo.txt files on Dropbox. The only missing piece was a way to see all my various projects' TODO files in one place.

![screenshot](https://raw.githubusercontent.com/kevinw/encasa/master/docs/static/screenshot.jpg)

## Setup

Put a file called `homepage.yaml` in your $HOME directory:

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

## Keyboard shortcuts

 * j - move down one task
 * k - move up one task
 * x - toggle task finished or unfinished
 * enter - follow first link in task
 * \D - archive finished tasks into done.txt files next to their respective todo.txt files
 * gg - go to the first task
 * G - go to the last task

## Rust libraries used

* [actix-web](http://actix.rs/) for the web api
* [askama](https://github.com/djc/askama) for type-safe simple HTML templating
* [humantime](https://github.com/tailhook/humantime) for parsing human readable times
* and of course, [serde](https://serde.rs/) for serializing
