function waitForMs(ms) {
    return new Promise(resolve => setTimeout(resolve, ms))
}


async function typeIntro(delay = 150) {
    const helloText = "Hello,";
    const nameText = "My name is Francisco Bruno;";
    const welcomeText = "Welcome,";
    const siteText = "to my <coding/> website:";

    let i = 0;

    while(i < helloText.length) {
        await this.waitForMs(delay);
        document.getElementById("hello-text").innerHTML += helloText[i];
        i++;
    }


    document.getElementById("hello-text").style.color = PinkControl;
    document.getElementById("hello-input").classList.remove("input-cursor");
    document.getElementById("name-input").classList.add("input-cursor");
    await this.waitForMs(delay / 2);

    i = 0;

    while(i < nameText.length) {
        await this.waitForMs(delay);
        document.getElementById("name-text").innerHTML += nameText[i];
        if( i === 7 ){
            document.getElementById("name-text").innerHTML = `My <span class="welcome-text" style='color: ${GreenClass};' > name </>`;
        }
        if( i === 20 ){
            document.getElementById("name-text").innerHTML = `My <span class="welcome-text" style='color: ${GreenClass};' > name </>`;
            document.getElementById("name-text").innerHTML += ` is `;
            document.getElementById("name-text").innerHTML += `<span class="welcome-text" style='color: ${BlueVar};' > Francisco </> `;
        }
        if( i === 25 ){
            document.getElementById("name-text").innerHTML = `My <span class="welcome-text" style='color: ${GreenClass};' > name </>`;
            document.getElementById("name-text").innerHTML += ` is `;
            document.getElementById("name-text").innerHTML += `<span class="welcome-text" style='color: ${BlueVar};' > Francisco Bruno</>`;
        }

        i++;
    }

    document.getElementById("name-input").classList.remove("input-cursor");
    document.getElementById("welcome-input").classList.add("input-cursor");
    await this.waitForMs(delay / 2);

    i = 0;

    while(i < welcomeText.length) {
        await this.waitForMs(delay);
        document.getElementById("welcome-text").innerHTML += welcomeText[i];
        i++;
    }

    document.getElementById("welcome-text").style.color = PinkControl;
    document.getElementById("welcome-input").classList.remove("input-cursor");
    document.getElementById("site-input").classList.add("input-cursor");
    await this.waitForMs(delay / 2);

    i = 0;

    while(i < siteText.length) {
        const idName = 'site-text';

        await this.waitForMs(delay);
        if( i < 7 || i > 14 ){
            document.getElementById(idName).innerHTML += siteText[i];
        }
        else if( i === 7 ){
            document.getElementById(idName).innerHTML = `to my <span class="welcome-text" style='color: ${GreySpecial};' >&lt;</>`
            document.getElementById(idName).innerHTML += `<span class="welcome-text" style='color: ${BlueDiv};' >c</>`
        }
        else if( i === 8 ){
            document.getElementById(idName).innerHTML = `to my <span class="welcome-text" style='color: ${GreySpecial};' >&lt;</>`
            document.getElementById(idName).innerHTML += `<span class="welcome-text" style='color: ${BlueDiv};' >co</>`
        }
        else if( i === 9 ){
            document.getElementById(idName).innerHTML = `to my <span class="welcome-text" style='color: ${GreySpecial};' >&lt;</>`
            document.getElementById(idName).innerHTML += `<span class="welcome-text" style='color: ${BlueDiv};' >cod</>`
        }
        else if( i === 10 ){
            document.getElementById(idName).innerHTML = `to my <span class="welcome-text" style='color: ${GreySpecial};' >&lt;</>`
            document.getElementById(idName).innerHTML += `<span class="welcome-text" style='color: ${BlueDiv};' >codi</>`
        }
        else if( i === 11 ){
            document.getElementById(idName).innerHTML = `to my <span class="welcome-text" style='color: ${GreySpecial};' >&lt;</>`
            document.getElementById(idName).innerHTML += `<span class="welcome-text" style='color: ${BlueDiv};' >codin</>`
        }
        else if( i === 12 ){
            document.getElementById(idName).innerHTML = `to my <span class="welcome-text" style='color: ${GreySpecial};' >&lt;</>`
            document.getElementById(idName).innerHTML += `<span class="welcome-text" style='color: ${BlueDiv};' >coding</>`
        }
        else if( i === 13 ){
            document.getElementById(idName).innerHTML = "to my &lt;coding/"
        }
        else if( i === 14 ){
            document.getElementById(idName).innerHTML = `to my <span class="welcome-text" style='color: ${GreySpecial};' >&lt;</>`
            document.getElementById(idName).innerHTML += `<span class="welcome-text" style='color: ${BlueDiv};' >coding</>`
            document.getElementById(idName).innerHTML += `<span class="welcome-text" style='color: ${GreySpecial};' >/&gt;</>`
        }

        i++;
    }

    return;
}