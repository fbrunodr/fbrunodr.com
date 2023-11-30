// default mode is get chat
set_get_chat();


async function get_chat() {
    const name = document.getElementById("name").value;
    const password = document.getElementById("password").value;

    const responseDiv = document.getElementById("response");

    fetch('/who_chat/get', {
        method: 'POST',
        headers: {
            'Accept': 'application/json',
            'Content-Type': 'application/json; charset=UTF-8'
        },
        body: JSON.stringify({ "name": name, "password": password })
    })
    .then((response) => {
        if (response.status == 200)
            responseDiv.style.backgroundColor = "rgba(0, 0, 0, 0.5)";
        else
            responseDiv.style.backgroundColor = "rgba(255, 0, 0, 0.4)";
        return response.text();
    })
    .then(data => {
        const time = new Date();
        responseDiv.innerText = `Response (${time.toLocaleTimeString()}):\n${data}`;
    });
}


async function post_chat() {
    const name = document.getElementById("name").value;
    const password = document.getElementById("password").value;
    const content = document.getElementById("content").value;

    const responseDiv = document.getElementById("response"); 

    fetch('/who_chat/post', {
        method: 'POST',
        headers: {
            'Accept': 'application/json',
            'Content-Type': 'application/json; charset=UTF-8'
        },
        body: JSON.stringify({ "name": name, "password": password, "content": content })
    })
    .then((response) => {
        if (response.status == 200)
            responseDiv.style.backgroundColor = "rgba(0, 255, 0, 0.4)";
        else
            responseDiv.style.backgroundColor = "rgba(255, 0, 0, 0.4)";
        return response.text();
    })
    .then(data => {
        const time = new Date();
        responseDiv.innerText = `Response (${time.toLocaleTimeString()}):\n${data}`;
    });
}


async function delete_chat() {
    const name = document.getElementById("name").value;
    const password = document.getElementById("password").value;

    const responseDiv = document.getElementById("response");

    fetch('/who_chat/delete', {
        method: 'POST',
        headers: {
            'Accept': 'application/json',
            'Content-Type': 'application/json; charset=UTF-8'
        },
        body: JSON.stringify({ "name": name, "password": password })
    })
    .then((response) => {
        if (response.status == 200)
            responseDiv.style.backgroundColor = "rgba(0, 255, 0, 0.5)";
        else
            responseDiv.style.backgroundColor = "rgba(255, 0, 0, 0.4)";
        return response.text();
    })
    .then(data => {
        const time = new Date();
        responseDiv.innerText = `Response (${time.toLocaleTimeString()}):\n${data}`;
    });
}


function set_get_chat() {
    const forms_wrapper = document.getElementById("forms-wrapper");
    forms_wrapper.innerHTML = `
        <form id="data-form" onsubmit="event.preventDefault(); return get_chat()">
            <div class=\"item\">
                <label for="name">Chat name:&nbsp;</label>
                <input type="text" id="name" name="name" required>
            </div>
            <div class=\"item\">
                <label for="password">Password:&nbsp&nbsp</label>
                <input type="password" id="password" name="password" required>
            </div>
            <input class="button" type="submit" value="Get Chat">
        </form>
    `;
	document.getElementById("get-switch").style = "color: black; background-color: #c71c63";
	document.getElementById("post-switch").style = "color: #c71c63; background-color: #36384c";
	document.getElementById("delete-switch").style = "color: crimson; background-color: #36384c";
}


function set_post_chat() {
    const forms_wrapper = document.getElementById("forms-wrapper");
    forms_wrapper.innerHTML = `
        <form id="data-form" onsubmit="event.preventDefault(); return post_chat()">
            <div class=\"item\">
                <label for="name">Chat name:&nbsp</label>
                <input type="text" id="name" name="name" required>
            </div>
            <div class=\"item\">
                <label for="password">Password:&nbsp&nbsp</label>
                <input type="password" id="password" name="password" required>
            </div>
            <div class=\"item\">
                <label for="content">Content:&nbsp</label>
                <textarea rows="4" cols="40" form="data-form" id="content">Type content here</textarea required>
            </div>
            <input class="button" type="submit" value="Post">
        </form>
    `;
	document.getElementById("get-switch").style = "color: #c71c63; background-color: #36384c";
	document.getElementById("post-switch").style = "color: black; background-color: #c71c63";
	document.getElementById("delete-switch").style = "color: crimson; background-color: #36384c";
}


function set_delete_chat() {
    const forms_wrapper = document.getElementById("forms-wrapper");
    forms_wrapper.innerHTML = `
        <form id="data-form" onsubmit="event.preventDefault(); return delete_chat()">
            <div class=\"item\">
                <label for="name">Chat name:&nbsp;</label>
                <input type="text" id="name" name="name" required>
            </div>
            <div class=\"item\">
                <label for="password">Password:&nbsp&nbsp</label>
                <input type="password" id="password" name="password" required>
            </div>
            <input class="button" type="submit" value="Delete Chat">
        </form>
    `;
	document.getElementById("get-switch").style = "color: #c71c63; background-color: #36384c";
	document.getElementById("post-switch").style = "color: #c71c63; background-color: #36384c";
	document.getElementById("delete-switch").style = "color: black; background-color: crimson";
}
