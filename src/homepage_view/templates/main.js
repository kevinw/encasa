"use strict";

function removeElement(array, elem) {
  const index = array.indexOf(elem);
  if (index > -1) {
    array.splice(index, 1);
  }
}

const activeRequests = [];

function postJSON(url, data, cb) {
  console.log("POST " + url + " " + JSON.stringify(data));
  function reqListener() {
    console.log(this.response);
  }
  const xhr = new XMLHttpRequest();
  xhr.addEventListener("load", reqListener);
  xhr.open("POST", url);

  xhr.setRequestHeader('Content-Type', 'application/json');

  activeRequests.push(xhr);

  xhr.onreadystatechange = function() {
    if (this.readyState != 4)
      return;

    removeElement(activeRequests, xhr);

    if (this.status == 200) {
        const res = JSON.parse(this.responseText);
        if (res == null)
            showNotification("Could not parse JSON: " + this.responseText);
        else if (cb)
            cb(res);
    } else {
      showNotification(this.responseText || "An error occurred.");
    }
  };

  xhr.send(JSON.stringify(data));
}

function archiveFinishedTasks() {
  postJSON("/actions/archive_finished", {}, function(res) {
    console.log(res);
    location.reload(); // maybe we can have the response show which hashes went away?
  });
}

function markTodo(hash, completed, cb) {
  postJSON("/todos", { hash: hash, completed: completed }, cb);
}

function clickTodo(e) {
  const node = e.target;
  if (node.nodeName !== "INPUT")
    return;

  const hash = node.getAttribute("value");
  const markFinished = node.checked;
  markTodo(hash, markFinished, function(res) {
    if (res.hash) {
      node.value = res.hash;
    }
  });
}

function getFocusedElement() {
  if (document.hasFocus() &&
      document.activeElement !== document.body &&
      document.activeElement !== document.documentElement)
  {
      return document.activeElement;
  }
}

function mod(n, m) {
  return ((n % m) + m) % m;
}

const keySequence = [];

function handleKeySequences(keyName) {
  keySequence.push(keyName);
  if (keySequence.length > 10) {
    keySequence.splice(0, keySequence.length - 10);
  }

  const len = keySequence.length;
  if (len > 1 && keySequence[len - 2] === "g" && keySequence[len - 1] === "g") {
    // vimlike: gg goes to top
    keySequence.length = 0;
    navigateToFirst();
    return false;
  } else if (len > 1 && keySequence[len - 2] === "\\" && keySequence[len - 1] === "D") {
    archiveFinishedTasks();
  }
}

function onKeyPress(event) {
  const keyName = event.key;
  if (handleKeySequences(keyName) === false)
    return false;

  switch (keyName) {
      case "j":
        navigateKeys(1);
        return false;
      case "k":
        navigateKeys(-1);
        return false;
      case "x":
      {
        const elem = getFocusedElement();
        if (elem.nodeName == "INPUT") {
          //elem.checked = !elem.checked;
            elem.click();
          return false;
        }
        break;
      }
      case "Enter":
      {
        // If in a TODO list, enter follows the link.
        const elem = getFocusedElement();
        if (elem.nodeName == "INPUT") {
          const links = [].slice.call(elem.parentElement.getElementsByTagName('a')).filter((e) => {
            const classes = e.classList;
            return !classes.contains("todo-context") && !classes.contains("todo-project");
          })
          if (links.length > 0) {
            links[0].click();
            return false;
          }
        }
        break;
      }
      case "G":
        navigateToLast();
        return false;
    default:
      console.log("key: " + keyName);
  }
}

var notification;

function showNotification(message) {
    console.error("notification: " + message);
    notification.getElementsByClassName("notification-message")[0].innerText = message;
    notification.style.display = "";
}

document.addEventListener("DOMContentLoaded", function() {
  const todoList = document.getElementById("todo_list");
  todoList.addEventListener("change", function(e) {
    const classList = e.target.parentElement.classList;
    if (e.target.checked)
      classList.add("todo-done");
    else
      classList.remove("todo-done");
    clickTodo(e);
  }, false);

  window.addEventListener("beforeunload", function (e) {
    if (activeRequests.length === 0)
      return;

    const confirmationMessage = "There are pending requests; are you sure you want to navigate away?";
    e.returnValue = confirmationMessage;     // Gecko, Trident, Chrome 34+
    return confirmationMessage;              // Gecko, WebKit, Chrome <34
  });

  document.addEventListener('keypress', onKeyPress);

  notification = document.getElementsByClassName("notification")[0];
  const deleteButton = notification.getElementsByClassName("delete")[0];
  deleteButton.addEventListener("click", function() {
      notification.style.display = "none";
  });
});

function _navigate(fn) {
  const elem = getFocusedElement();
  const allNavigable = [].slice.call(document.getElementsByClassName("navigable-elem"));
  const currentIndex = allNavigable.indexOf(elem);
  let newIndex = fn(allNavigable, currentIndex);
  if (newIndex < -1)
    newIndex = -1;
  allNavigable[mod(newIndex,  allNavigable.length)].focus();
}

function navigateKeys(delta) {
  _navigate((elems, idx) => idx + delta);
}

function navigateToFirst() {
  _navigate((elems, idx) => 0);
}

function navigateToLast() {
  _navigate((elems, idx) => elems.length - 1);
}

