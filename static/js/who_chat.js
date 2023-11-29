async function get_chat_data() {
    const name = document.getElementById("name").value;
    const password = document.getElementById("password").value;

    fetch('/who_chat/get', {
        method: 'POST',
        headers: {
            'Accept': 'application/json',
            'Content-Type': 'application/json; charset=UTF-8'
        },
        body: JSON.stringify({ "name": name, "password": password })
    })
    .then((response) => response.text())
    .then(data => {
        console.log(data);
    })
}
