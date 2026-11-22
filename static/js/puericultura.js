// Puericultura: the list with its traffic light, the registration form and the consultation checklists.
// Ages, the next return and who is late all come computed from the server; this file only shows them.

// The guide text of each consultation, keyed by the age in months at which it is due (0 = first week), the same
// keys the server answers with: the SBP routine of one consultation in the 1st or 2nd week, monthly up to 6 months
// and every three months up to 2 years. It is a static reminder, based on the Caderneta da Criança and the Caderno
// de Atenção Básica nº 33 (Ministério da Saúde) and on SBP recommendations. Edit the text here.
const CHECKLISTS = {
  0: {
    titulo: "1ª ou 2ª semana",
    itens: [
      "Checar os testes de triagem neonatal: pezinho (coleta entre o 3º e o 5º dia), orelhinha, olhinho (reflexo vermelho), coraçãozinho e linguinha — feitos? resultados? encaminhar alterações",
      "Conferir vacinas ao nascer: BCG e hepatite B",
      "Avaliar a mamada (pega e posição) e apoiar o aleitamento materno exclusivo",
      "Peso: perda de até 10% do peso de nascimento é esperada; recuperação até cerca do 10º–14º dia",
      "Avaliar icterícia, coto umbilical, hidratação e eliminações",
      "Orientar sono seguro (barriga para cima) e sinais de perigo que exigem atendimento imediato",
      "Avaliar a saúde da mãe no puerpério e a rede de apoio da família",
    ],
  },
  1: {
    titulo: "1 mês",
    itens: [
      "Triagem neonatal: conferir resultados que ficaram pendentes na primeira consulta e encaminhar alterações",
      "Conferir BCG e hepatite B; orientar as vacinas dos 2 meses",
      "Aleitamento materno exclusivo: dificuldades, ganho de peso (esperado cerca de 20–30 g/dia)",
      "Exame físico completo: reflexo vermelho, ausculta cardíaca, quadris (Ortolani/Barlow), testículos nos meninos",
      "Marcos: fixa o olhar no rosto, reage ao som, eleva a cabeça por instantes de bruços",
    ],
  },
  2: {
    titulo: "2 meses",
    itens: [
      "Conferir as vacinas dos 2 meses: pentavalente, VIP, pneumocócica 10 e rotavírus",
      "Orientar a meningocócica C aos 3 e aos 5 meses",
      "Aleitamento materno exclusivo; orientar a mãe sobre ordenha e volta ao trabalho",
      "Marcos: sorriso social, emite sons, acompanha objetos com o olhar, sustenta a cabeça por mais tempo de bruços",
      "Orientar sobre cólicas e choro, sono seguro e prevenção de acidentes (quedas, transporte no bebê conforto)",
    ],
  },
  3: {
    titulo: "3 meses",
    itens: [
      "Conferir a 1ª dose da meningocócica C (3 meses) e as vacinas dos 2 meses",
      "Aleitamento materno exclusivo; planejar a volta da mãe ao trabalho (ordenha e armazenamento do leite)",
      "Marcos: sustenta a cabeça, sorri e vocaliza em resposta, abre as mãos, acompanha objetos em 180°",
      "Orientar sono seguro e prevenção de quedas",
    ],
  },
  4: {
    titulo: "4 meses",
    itens: [
      "Conferir as vacinas dos 4 meses (2ª dose de pentavalente, VIP, pneumocócica 10 e rotavírus) e a meningocócica C dos 3 meses",
      "Manter aleitamento materno exclusivo: não introduzir água, chás ou outros alimentos antes dos 6 meses",
      "Marcos: sustenta a cabeça, leva as mãos à linha média, agarra objetos, ri alto, vira-se em direção ao som",
      "Orientar prevenção de acidentes: quedas do trocador e da cama, objetos pequenos",
    ],
  },
  5: {
    titulo: "5 meses",
    itens: [
      "Conferir a 2ª dose da meningocócica C (5 meses) e as vacinas dos 4 meses",
      "Manter aleitamento materno exclusivo; preparar a família para a introdução alimentar aos 6 meses",
      "Marcos: rola, apoia-se nos antebraços de bruços, leva objetos à boca, ri alto, reconhece os cuidadores",
      "Orientar prevenção de acidentes: objetos pequenos, quedas, não usar andador",
    ],
  },
  6: {
    titulo: "6 meses",
    itens: [
      "Orientar a introdução da alimentação complementar, mantendo o leite materno; sem açúcar e sem ultraprocessados até os 2 anos",
      "Conferir as vacinas dos 6 meses (3ª dose de pentavalente e VIP), meningocócica C dos 5 meses, influenza e covid-19",
      "Vitamina A: megadose de 100.000 UI entre 6 e 11 meses, onde o programa se aplica",
      "Marcos: senta com apoio, rola, transfere objetos de uma mão para a outra, balbucia",
      "Saúde bucal: higiene a partir do primeiro dente, creme dental com flúor em quantidade mínima",
    ],
  },
  9: {
    titulo: "9 meses",
    itens: [
      "Conferir a vacina de febre amarela (9 meses) e o esquema de covid-19",
      "Alimentação: evoluir a consistência (amassado → pedaços), oferecer carnes e leguminosas, comer junto com a família",
      "Marcos: senta sem apoio, arrasta-se ou engatinha, duplica sílabas (\"mama\", \"papa\"), estranha desconhecidos, pega em pinça",
      "Orientar prevenção de acidentes: engasgo, queimaduras, intoxicações, tomadas e escadas",
      "Orientar: nenhuma exposição a telas antes dos 2 anos",
    ],
  },
  12: {
    titulo: "12 meses",
    itens: [
      "Conferir as vacinas dos 12 meses: tríplice viral, reforço da pneumocócica 10 e reforço meningocócico",
      "Orientar as vacinas dos 15 meses: DTP, VIP, hepatite A e tetraviral",
      "Alimentação da família, com variedade; manter o leite materno; sem açúcar e sem ultraprocessados",
      "Considerar hemograma e ferritina para rastrear anemia",
      "Vitamina A: megadose de 200.000 UI a cada 6 meses a partir dos 12 meses, onde o programa se aplica",
      "Marcos: fica em pé e anda com apoio, fala ao menos uma palavra com sentido, aponta, dá tchau, imita gestos",
    ],
  },
  15: {
    titulo: "15 meses",
    itens: [
      "Conferir as vacinas dos 15 meses: DTP, VIP, hepatite A e tetraviral",
      "Marcos: anda sozinho ou com pouco apoio, fala 2 a 3 palavras, aponta o que quer, entende ordens simples, empilha 2 cubos",
      "Alimentação da família, come sozinho com as mãos e começa a usar colher; copo no lugar da mamadeira",
      "Saúde bucal: escovação pelos pais, evitar mamadeira noturna",
      "Orientar prevenção de acidentes: quedas, queimaduras, afogamento, produtos de limpeza e medicamentos",
    ],
  },
  18: {
    titulo: "18 meses",
    itens: [
      "Aplicar o M-CHAT-R/F (rastreio de transtorno do espectro autista, dos 16 aos 30 meses)",
      "Conferir as vacinas dos 15 meses: DTP, VIP, hepatite A e tetraviral",
      "Marcos: anda sozinho, fala pelo menos 3 a 6 palavras, aponta para mostrar o que quer, brinca de faz de conta, rabisca",
      "Alimentação variada, sem ultraprocessados",
      "Orientar: sem telas antes dos 2 anos; ler e brincar junto todos os dias",
      "Saúde bucal: escovação pelos pais, evitar mamadeira noturna",
      "Orientar prevenção de acidentes: afogamento, quedas, produtos de limpeza e medicamentos fora do alcance",
    ],
  },
  21: {
    titulo: "21 meses",
    itens: [
      "Marcos: vocabulário em expansão (cerca de 20 palavras ou mais), começa a juntar duas palavras, sobe degraus com apoio, imita tarefas da casa",
      "Conferir o resultado do M-CHAT-R/F; aplicar se ainda não foi aplicado",
      "Conferir a caderneta de vacinação e as campanhas",
      "Alimentação variada, sem ultraprocessados; sono e rotina",
      "Orientar: sem telas antes dos 2 anos; birras e limites; sinais de prontidão para o desfralde",
    ],
  },
  24: {
    titulo: "24 meses",
    itens: [
      "Marcos: junta duas palavras, corre, chuta bola, sobe degraus, aponta figuras e partes do corpo, tira peças de roupa",
      "Aplicar o M-CHAT-R/F se ainda não foi aplicado",
      "Conferir a caderneta de vacinação completa até os 15 meses e as campanhas",
      "Avaliar o IMC para a idade nas curvas; alimentação e sono",
      "Orientar: telas no máximo 1 hora por dia, sempre com um adulto; desfralde só com sinais de prontidão",
      "Programar o seguimento anual a partir dos 2 anos",
    ],
  },
};

const EM_TODA_CONSULTA = [
  "Pesar, medir comprimento e perímetro cefálico; marcar nas curvas da Caderneta da Criança",
  "Avaliar o desenvolvimento pelos marcos da Caderneta (no prematuro, pela idade corrigida)",
  "Conferir a caderneta de vacinação pelo calendário vigente do PNI",
  "Prescrever ou conferir a suplementação de vitamina D e ferro (quadro desta criança, acima)",
  "Registrar a consulta e agendar o retorno",
];

const MARCOS = Object.keys(CHECKLISTS).map(Number).sort((a, b) => a - b);

let criancas = [];
let hoje = "";
const filtros = { busca: "", acs: "", atrasadas: false };
let editandoId = null;
let previaTimer = null;
let previaPedido = 0;

const $ = (id) => document.getElementById(id);

// Builds an element. Text only ever enters through textContent, so a name is never read as markup.
function el(tag, attrs, ...filhos) {
  const node = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs || {})) {
    if (v === false || v == null) continue;
    if (k === "class") node.className = v;
    else if (k.startsWith("on")) node.addEventListener(k.slice(2), v);
    else node.setAttribute(k, v === true ? "" : v);
  }
  for (const f of filhos.flat()) {
    if (f == null || f === false) continue;
    node.append(f instanceof Node ? f : document.createTextNode(String(f)));
  }
  return node;
}

// Replaces what is inside a node, leaving out the pieces a condition turned off.
function put(alvo, ...filhos) {
  alvo.replaceChildren(...filhos.flat().filter((f) => f != null && f !== false));
}

// dd/mm/aaaa from an ISO yyyy-mm-dd (built from the parts, so no timezone shifts it)
function fmtDate(iso) {
  if (!iso) return "";
  const p = iso.split("-");
  return p.length === 3 ? `${p[2]}/${p[1]}/${p[0]}` : iso;
}

function plural(n, um, muitos) {
  return `${n} ${n === 1 ? um : muitos}`;
}

function fmtIdade(idade) {
  const { meses, dias } = idade;
  if (meses >= 12) {
    const anos = Math.floor(meses / 12);
    const resto = meses % 12;
    return plural(anos, "ano", "anos") + (resto ? " e " + plural(resto, "mês", "meses") : "");
  }
  if (meses >= 1) return plural(meses, "mês", "meses") + (dias ? " e " + plural(dias, "dia", "dias") : "");
  return plural(dias, "dia", "dias");
}

// "32 semanas e 4 dias"; the days are left out when there are none.
function fmtSemanasDias(totalDias) {
  const semanas = Math.floor(totalDias / 7), dias = totalDias % 7;
  if (!semanas) return plural(dias, "dia", "dias");
  return plural(semanas, "semana", "semanas") + (dias ? " e " + plural(dias, "dia", "dias") : "");
}

function igEmDias(c) {
  return c.ig_semanas * 7 + (c.ig_dias || 0);
}

function fmtPeso(gramas) {
  return gramas.toLocaleString("pt-BR") + " g";
}

// The unit's supplementation protocol said for this child today. Which group and phase the child is in comes
// from the server; the wording of the protocol is here.
function linhasSuplementacao(c) {
  const s = c.suplementacao;
  if (!s) return [];
  const linhas = [["Vitamina D", [`${s.vitamina_d_ui} UI, 1 vez ao dia, todos os dias (400 UI até 1 ano; 600 UI de 1 a 2 anos).`]]];
  const f = s.ferro;
  const ate = f && f.ate ? fmtDate(f.ate) : "";
  const formula = "Se recebe fórmula infantil, descontar o ferro da fórmula da dose (pode ajustar ou dispensar o suplemento).";

  if (!f) {
    const falta = [c.ig_semanas == null && "a idade gestacional", c.peso_nascimento == null && "o peso ao nascer"].filter(Boolean).join(" e ");
    linhas.push(["Ferro", [`Informe ${falta} no cadastro para ver o esquema desta criança.`]]);
  } else if (f.grupo === "termo") {
    const dose = "10 a 12,5 mg/dia de ferro elementar (dose fixa, não depende do peso)";
    const agora = {
      antes: `Ainda não: o 1º ciclo começa aos 6 meses, em ${ate}.`,
      ciclo1: `1º ciclo (6 aos 9 meses): ${dose}, uso diário até ${ate}.`,
      pausa: `Pausa (9 aos 12 meses): sem suplementação até ${ate}, quando começa o 2º ciclo.`,
      ciclo2: `2º ciclo (12 aos 15 meses): ${dose}, uso diário até ${ate}; a profilaxia termina ao fim deste ciclo.`,
      encerrada: "Profilaxia encerrada: os dois ciclos (6–9 e 12–15 meses) já passaram.",
    }[f.fase];
    const notas = [agora, "A termo, com peso adequado ao nascer: esquema em dois ciclos, 6–9 meses e 12–15 meses, com pausa entre eles."];
    if (f.fase === "antes") notas.push("Se recebe leite de vaca integral antes de 1 ano (contraindicado), antecipar o início para os 4 meses, junto com a introdução alimentar.");
    if (f.fase !== "encerrada") notas.push(formula);
    linhas.push(["Ferro", notas]);
  } else {
    const porque = [c.prematuro && "prematuro", c.peso_nascimento != null && c.peso_nascimento < 2500 && "baixo peso ao nascer"].filter(Boolean).join(" e ");
    const tabela = "1.500 a 2.500 g: 2 mg/kg/dia · 1.000 a 1.500 g: 3 mg/kg/dia · menos de 1.000 g: 4 mg/kg/dia";
    const dose1 = f.mg_kg
      ? `${f.mg_kg} mg/kg/dia de ferro elementar (peso ao nascer: ${fmtPeso(c.peso_nascimento)})`
      : `dose pelo peso ao nascer, que não está no cadastro (${tabela})`;
    const agora = {
      antes: `Ainda não: começa aos 30 dias de vida, em ${ate}, com ${dose1}.`,
      primeiro_ano: `1º ano de vida: ${dose1}, uso diário contínuo. A partir de ${ate} passa a 1 mg/kg/dia.`,
      segundo_ano: `2º ano de vida: 1 mg/kg/dia de ferro elementar, pelo peso atual, uso diário contínuo e sem pausas até os 24 meses (${ate}).`,
    }[f.fase];
    const notas = [agora, `${porque.charAt(0).toUpperCase() + porque.slice(1)}: profilaxia precoce e contínua, dos 30 dias aos 24 meses.`];
    if (c.prematuro && c.peso_nascimento >= 2500 && f.fase !== "segundo_ano") notas.push("O protocolo só traz doses até 2.500 g de peso ao nascer; para este prematuro foi usada a da faixa mais alta (2 mg/kg/dia). Confirme.");
    if (c.prematuro && f.fase !== "segundo_ano") notas.push("Prematuro que recebeu mais de 100 mL de concentrado de hemácias na internação: avaliar individualmente, podendo adiar o início.");
    notas.push(formula);
    linhas.push(["Ferro", notas]);
  }
  return linhas;
}

function fmtCorrigida(c) {
  if (!c.prematuro) return null;
  return c.idade_corrigida ? fmtIdade(c.idade_corrigida) : "ainda não chegou às 40 semanas";
}

function situacao(c) {
  if (c.atrasada) {
    return c.ultima_consulta
      ? `Atrasada há ${plural(c.dias_atraso, "dia", "dias")}`
      : `Atrasada há ${plural(c.dias_atraso, "dia", "dias")} · sem consulta registrada`;
  }
  return c.proximo_retorno ? "No prazo" : "Acompanhamento concluído";
}

function retorno(c) {
  if (!c.proximo_retorno) return "Consulta dos 24 meses feita";
  return `${fmtDate(c.proximo_retorno)} (consulta de ${CHECKLISTS[c.proxima_consulta].titulo})`;
}

function showMsg(text, ok) {
  const m = $("msg");
  m.textContent = text;
  m.className = "msg " + (ok ? "ok" : "err");
  if (text && ok) setTimeout(() => { if (m.textContent === text) { m.textContent = ""; m.className = "msg"; } }, 4000);
}

async function api(path, method, body) {
  const opts = { method, headers: {} };
  if (body !== undefined) {
    opts.headers["Content-Type"] = "application/json";
    opts.body = JSON.stringify(body);
  }
  const res = await fetch(path, opts);
  if (res.status === 401) {
    window.location.href = "/puericultura";
    throw new Error("sessão encerrada");
  }
  if (!res.ok) throw new Error(await res.text());
  return res;
}

async function load() {
  try {
    const dados = await (await api("/puericultura/api/criancas", "GET")).json();
    criancas = dados.criancas;
    hoje = dados.hoje;
    render();
  } catch (e) {
    showMsg("Erro ao carregar: " + e.message, false);
  }
}


// ----------------------------- List -----------------------------

// Late children first, the longest wait on top; then the ones on time, by the nearest return.
function ordenar(a, b) {
  if (a.atrasada !== b.atrasada) return a.atrasada ? -1 : 1;
  if (a.atrasada) return b.dias_atraso - a.dias_atraso || a.nome.localeCompare(b.nome, "pt-BR");
  const ra = a.proximo_retorno || "9999", rb = b.proximo_retorno || "9999";
  return ra.localeCompare(rb) || a.nome.localeCompare(b.nome, "pt-BR");
}

function semAcento(s) {
  return s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
}

function visiveis() {
  const busca = semAcento(filtros.busca.trim());
  return criancas
    .filter((c) => !filtros.acs || c.acs === filtros.acs)
    .filter((c) => !filtros.atrasadas || c.atrasada)
    .filter((c) => !busca || semAcento(c.nome).includes(busca) || (c.registro || "").includes(busca))
    .sort(ordenar);
}

function renderResumo() {
  const doAcs = criancas.filter((c) => !filtros.acs || c.acs === filtros.acs);
  const atrasadas = doAcs.filter((c) => c.atrasada).length;
  const cartao = (rotulo, valor, classe) =>
    el("div", { class: "cartao " + (classe || "") }, el("div", { class: "lbl" }, rotulo), el("div", { class: "val" }, valor));
  $("resumo").replaceChildren(
    cartao(filtros.acs ? "Crianças de " + filtros.acs : "Crianças", doAcs.length),
    cartao("No prazo", doAcs.length - atrasadas, "verde"),
    cartao("Atrasadas", atrasadas, "vermelho"),
  );
}

function renderFiltroAcs() {
  const nomes = [...new Set(criancas.map((c) => c.acs))].sort((a, b) => a.localeCompare(b, "pt-BR"));
  if (filtros.acs && !nomes.includes(filtros.acs)) filtros.acs = "";
  $("filtro-acs").replaceChildren(
    el("option", { value: "" }, "Todos os ACS"),
    ...nomes.map((n) => el("option", { value: n, selected: n === filtros.acs }, n)),
  );
  $("acs-list").replaceChildren(...nomes.map((n) => el("option", { value: n })));
}

function linha(c) {
  const corrigida = fmtCorrigida(c);
  return el("li", {},
    el("button", { type: "button", class: "crianca " + (c.atrasada ? "vermelho" : "verde"), onclick: () => abrirFicha(c.id) },
      el("span", { class: "luz", "aria-hidden": "true" }),
      el("span", { class: "quem" },
        el("span", { class: "nome" }, c.nome),
        el("span", { class: "detalhe" }, (c.registro ? `Reg. ${c.registro} · ` : "") + fmtIdade(c.idade) + (corrigida ? ` (corrigida: ${corrigida})` : "") + " · ACS " + c.acs),
      ),
      el("span", { class: "quando" },
        el("span", { class: "selo" }, situacao(c)),
        el("span", { class: "detalhe" }, c.proximo_retorno ? "Retorno: " + retorno(c) : retorno(c)),
      ),
    ),
  );
}

function render() {
  renderFiltroAcs();
  renderResumo();
  const btn = $("filtro-atrasadas");
  btn.setAttribute("aria-pressed", String(filtros.atrasadas));
  btn.classList.toggle("ativo", filtros.atrasadas);

  renderAgenda();
  renderPrematuros();

  const lista = visiveis();
  $("lista").replaceChildren(...lista.map(linha));
  const vazio = $("vazio");
  vazio.hidden = lista.length > 0;
  if (!lista.length) {
    vazio.textContent = !criancas.length
      ? "Nenhuma criança cadastrada ainda."
      : filtros.atrasadas && !filtros.busca
        ? "Nenhuma criança atrasada" + (filtros.acs ? " para " + filtros.acs : "") + "."
        : "Nenhuma criança encontrada com esses filtros.";
  }
}


// ----------------------------- Schedule -----------------------------

const MESES = ["Janeiro", "Fevereiro", "Março", "Abril", "Maio", "Junho", "Julho", "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro"];
const mesesAbertos = new Set();

// Every month from this one to the last with a consultation, each with the consultations that fall in it.
// The schedule is the unit's, so it counts every child whatever the list is filtered by.
function mesesDaAgenda() {
  const porMes = new Map();
  for (const c of criancas) {
    for (const p of c.agenda || []) {
      const mes = p.data.slice(0, 7);
      if (!porMes.has(mes)) porMes.set(mes, []);
      porMes.get(mes).push({ c, p });
    }
  }
  if (!porMes.size) return [];
  const ultimo = [...porMes.keys()].sort().pop();
  const meses = [];
  let [ano, mes] = hoje.split("-").map(Number);
  for (;;) {
    const chave = `${ano}-${String(mes).padStart(2, "0")}`;
    if (chave > ultimo) break;
    const consultas = (porMes.get(chave) || []).sort((a, b) =>
      (b.p.em_atraso - a.p.em_atraso) || a.p.data.localeCompare(b.p.data) || a.c.nome.localeCompare(b.c.nome, "pt-BR"));
    meses.push({ chave, ano, mes, consultas });
    if (++mes > 12) { mes = 1; ano++; }
  }
  return meses;
}

function renderAgenda() {
  const meses = mesesDaAgenda();
  if (!meses.length) {
    $("agenda").replaceChildren(el("p", { class: "vazio" }, "Nenhuma consulta prevista."));
    return;
  }
  const maior = Math.max(...meses.map((m) => m.consultas.length));
  $("agenda").replaceChildren(...meses.map((m) => {
    const atraso = m.consultas.filter((x) => x.p.em_atraso).length;
    const previstas = m.consultas.length - atraso;
    const partes = atraso ? `${plural(previstas, "prevista", "previstas")} + ${atraso} em atraso` : "";
    return el("details", { class: "mes", open: mesesAbertos.has(m.chave),
        ontoggle: (e) => { if (e.target.open) mesesAbertos.add(m.chave); else mesesAbertos.delete(m.chave); } },
      el("summary", {},
        el("span", { class: "mes-nome" }, `${MESES[m.mes - 1]} de ${m.ano}`),
        el("span", { class: "mes-vagas" }, el("strong", {}, m.consultas.length), m.consultas.length === 1 ? " vaga" : " vagas"),
        el("span", { class: "mes-barra", "aria-hidden": "true" },
          el("span", { class: "previstas", style: `width:${(100 * previstas) / maior}%` }),
          el("span", { class: "atraso", style: `width:${(100 * atraso) / maior}%` })),
        partes && el("span", { class: "detalhe mes-partes" }, partes),
      ),
      m.consultas.length
        ? el("ul", { class: "mes-lista" }, m.consultas.map(({ c, p }) =>
            el("li", {},
              el("button", { type: "button", class: "mes-crianca", onclick: () => abrirFicha(c.id) },
                el("span", { class: "nome" }, c.nome),
                el("span", { class: "detalhe" }, p.em_atraso
                  ? `Em atraso: agendar agora · consulta de ${CHECKLISTS[p.marco].titulo} · ACS ${c.acs}`
                  : `${fmtDate(p.data)} · consulta de ${CHECKLISTS[p.marco].titulo} · ACS ${c.acs}`)))))
        : el("p", { class: "detalhe mes-nada" }, "Nenhuma consulta prevista neste mês."),
    );
  }));
}

const ABAS = ["lista", "agenda", "prematuros"];

function mostrarAba(nome) {
  for (const aba of ABAS) {
    const ativa = aba === nome;
    $("sec-" + aba).hidden = !ativa;
    $("aba-" + aba).classList.toggle("ativa", ativa);
    $("aba-" + aba).setAttribute("aria-selected", String(ativa));
  }
}


// ----------------------------- Premature children -----------------------------

// The premature children, the most premature first, each with the corrected age beside the age since birth.
function renderPrematuros() {
  const prematuros = criancas.filter((c) => c.prematuro)
    .sort((a, b) => igEmDias(a) - igEmDias(b) || a.nome.localeCompare(b.nome, "pt-BR"));
  $("aba-prematuros").textContent = prematuros.length ? `Prematuros (${prematuros.length})` : "Prematuros";
  if (!prematuros.length) {
    $("prematuros").replaceChildren(el("li", { class: "vazio" },
      "Nenhuma criança prematura cadastrada. Preencha a idade gestacional no cadastro: abaixo de 37 semanas a criança aparece aqui."));
    return;
  }
  $("prematuros").replaceChildren(...prematuros.map((c) => {
    return el("li", {},
      el("button", { type: "button", class: "crianca prematuro", onclick: () => abrirFicha(c.id) },
        el("span", { class: "quem" },
          el("span", { class: "nome" }, c.nome),
          el("span", { class: "detalhe" }, `Nasceu com ${fmtSemanasDias(igEmDias(c))} (${fmtSemanasDias(280 - igEmDias(c))} antes do termo) em ${fmtDate(c.nascimento)} · ACS ${c.acs}`),
        ),
        el("span", { class: "idades" },
          el("span", { class: "idade-item" }, el("span", { class: "detalhe" }, "Idade corrigida"), el("strong", {}, fmtCorrigida(c))),
          el("span", { class: "idade-item" }, el("span", { class: "detalhe" }, "Idade cronológica"), el("span", {}, fmtIdade(c.idade))),
        ),
      ),
    );
  }));
}


// ----------------------------- Modals -----------------------------

let focoAnterior = null;

function abrirModal(id) {
  focoAnterior = document.activeElement;
  $(id).hidden = false;
  document.body.classList.add("modal-aberto");
}

function fecharModais() {
  for (const m of document.querySelectorAll(".modal-overlay")) m.hidden = true;
  document.body.classList.remove("modal-aberto");
  if (focoAnterior && document.contains(focoAnterior)) focoAnterior.focus();
  focoAnterior = null;
}


// ----------------------------- Child sheet and checklist -----------------------------

function renderChecklist(alvo, marco) {
  const lista = (itens) => el("ul", { class: "checklist" },
    itens.map((texto) => el("li", {}, el("label", {}, el("input", { type: "checkbox" }), el("span", {}, texto)))));
  alvo.replaceChildren(
    lista(CHECKLISTS[marco].itens),
    el("h4", {}, "Em toda consulta"),
    lista(EM_TODA_CONSULTA),
  );
}

function abrirFicha(id) {
  const c = criancas.find((x) => x.id === id);
  if (!c) return;
  $("ficha-nome").textContent = c.nome;

  const corrigida = fmtCorrigida(c);
  const dado = (rotulo, valor) => el("div", { class: "dado" }, el("dt", {}, rotulo), el("dd", {}, valor));
  const dados = el("dl", { class: "dados" },
    dado("Situação", el("span", { class: "selo " + (c.atrasada ? "vermelho" : "verde") }, situacao(c))),
    dado("Próximo retorno", retorno(c)),
    dado("Nº de registro", c.registro || "não informado"),
    dado("Idade", fmtIdade(c.idade) + ` (nasc. ${fmtDate(c.nascimento)})`),
    corrigida && dado("Idade corrigida", corrigida),
    dado("Idade gestacional", c.ig_semanas == null ? "não informada" : fmtSemanasDias(igEmDias(c)) + (c.prematuro ? " (prematuro)" : "")),
    dado("Peso ao nascer", c.peso_nascimento == null ? "não informado" : fmtPeso(c.peso_nascimento) + (c.peso_nascimento < 2500 ? " (baixo peso)" : "")),
    dado("ACS responsável", c.acs),
    dado("Última consulta", c.ultima_consulta ? fmtDate(c.ultima_consulta) : "sem registro"),
  );

  const corpoChecklist = el("div", {});
  const seletor = el("select", { id: "ficha-faixa", "aria-label": "Faixa etária do checklist", onchange: (e) => renderChecklist(corpoChecklist, Number(e.target.value)) },
    MARCOS.map((m) => el("option", { value: m, selected: m === c.faixa }, CHECKLISTS[m].titulo + (m === c.faixa ? " (faixa atual)" : ""))));
  renderChecklist(corpoChecklist, c.faixa);

  const fichaMsg = el("div", { class: "msg err", role: "alert" });
  const dataConsulta = el("input", { type: "date", id: "ficha-data", value: hoje, max: hoje, min: c.nascimento, "aria-label": "Data da consulta" });
  const registrar = el("button", { type: "button", class: "btn primary", onclick: async () => {
    registrar.disabled = true;
    try {
      await api("/puericultura/api/criancas/consulta", "POST", { id: c.id, data: dataConsulta.value });
      fecharModais();
      showMsg("Consulta registrada.", true);
      await load();
    } catch (e) {
      fichaMsg.textContent = e.message;
      registrar.disabled = false;
    }
  } }, "Registrar consulta");

  put($("ficha-corpo"),
    dados,
    el("div", { class: "suplementacao" },
      el("h3", {}, "Suplementação"),
      linhasSuplementacao(c).map(([titulo, notas]) => el("div", { class: "suplemento" },
        el("h4", {}, titulo),
        el("p", { class: "agora" }, notas[0]),
        notas.slice(1).map((n) => el("p", { class: "nota" }, n))))),
    el("div", { class: "checklist-head" }, el("h3", {}, "Checklist da consulta"), seletor),
    c.prematuro && el("p", { class: "nota" }, "Prematuro: avalie crescimento e desenvolvimento pela idade corrigida; vacinas seguem a idade cronológica."),
    corpoChecklist,
    el("p", { class: "nota" }, "Texto guia: não substitui a avaliação clínica. As marcações não são salvas."),
    el("div", { class: "registrar" }, el("label", { for: "ficha-data" }, "Consulta feita em"), dataConsulta, registrar),
    fichaMsg,
    el("div", { class: "modal-actions" },
      el("button", { type: "button", class: "btn danger", onclick: () => excluir(c) }, "Excluir"),
      el("button", { type: "button", class: "btn", onclick: () => { fecharModais(); abrirForm(c); } }, "Editar cadastro"),
    ),
  );
  abrirModal("ficha-modal");
}

async function excluir(c) {
  if (!window.confirm(`Excluir o cadastro de ${c.nome}? Isso não pode ser desfeito.`)) return;
  try {
    await api("/puericultura/api/criancas/delete", "POST", { id: c.id });
    fecharModais();
    showMsg("Cadastro excluído.", true);
    await load();
  } catch (e) {
    showMsg("Erro ao excluir: " + e.message, false);
  }
}


// ----------------------------- Registration form -----------------------------

function lerForm() {
  const ig = $("f-ig").value.trim();
  const igDias = $("f-ig-dias").value.trim();
  return {
    registro: $("f-registro").value.trim() || null,
    nome: $("f-nome").value,
    nascimento: $("f-nascimento").value,
    ig_semanas: ig === "" ? null : Number(ig),
    ig_dias: igDias === "" ? null : Number(igDias),
    peso_nascimento: $("f-peso").value.trim() === "" ? null : Number($("f-peso").value),
    acs: $("f-acs").value,
    ultima_consulta: $("f-ultima").value || null,
  };
}

function abrirForm(c) {
  editandoId = c ? c.id : null;
  $("form-titulo").textContent = c ? "Editar cadastro" : "Cadastrar criança";
  $("f-registro").value = c && c.registro ? c.registro : "";
  $("f-nome").value = c ? c.nome : "";
  $("f-nascimento").value = c ? c.nascimento : "";
  $("f-ig").value = c && c.ig_semanas != null ? c.ig_semanas : "";
  $("f-ig-dias").value = c && c.ig_semanas != null ? c.ig_dias || 0 : "";
  $("f-peso").value = c && c.peso_nascimento != null ? c.peso_nascimento : "";
  $("f-acs").value = c ? c.acs : filtros.acs;
  $("f-ultima").value = c && c.ultima_consulta ? c.ultima_consulta : "";
  $("f-nascimento").max = hoje;
  $("f-ultima").max = hoje;
  $("form-msg").textContent = "";
  $("previa").replaceChildren();
  abrirModal("form-modal");
  $("f-registro").focus();
  if (c) previa();
}

// What the spreadsheet formula did: as the dates are typed, show the corrected age and the next return.
function previa() {
  clearTimeout(previaTimer);
  previaTimer = setTimeout(async () => {
    const dados = lerForm();
    const alvo = $("previa");
    const pedido = ++previaPedido;
    if (!dados.nascimento) { alvo.replaceChildren(); return; }
    // The preview depends only on the dates and the gestational age; the texts may still be empty.
    const corpo = { ...dados, nome: dados.nome.trim() || "-", acs: dados.acs.trim() || "-" };
    // A gestational age still half typed is left out of the preview rather than refused.
    if (!Number.isInteger(corpo.ig_semanas)) corpo.ig_semanas = null;
    if (corpo.ig_semanas === null || !Number.isInteger(corpo.ig_dias)) corpo.ig_dias = null;
    corpo.peso_nascimento = null;
    try {
      const c = await (await api("/puericultura/api/calcular", "POST", corpo)).json();
      if (pedido !== previaPedido) return;
      const corrigida = fmtCorrigida(c);
      put(alvo,
        el("div", {}, el("strong", {}, "Idade: "), fmtIdade(c.idade)),
        corrigida && el("div", {}, el("strong", {}, "Idade corrigida: "), corrigida),
        el("div", {}, el("strong", {}, "Próximo retorno: "), retorno(c)),
        el("div", {}, el("span", { class: "selo " + (c.atrasada ? "vermelho" : "verde") }, c.atrasada ? `Atrasada há ${plural(c.dias_atraso, "dia", "dias")}` : situacao(c))),
      );
    } catch (e) {
      if (pedido === previaPedido) alvo.replaceChildren();
    }
  }, 250);
}

async function salvar(evento) {
  evento.preventDefault();
  const dados = lerForm();
  const msg = $("form-msg");
  if (dados.peso_nascimento !== null && !Number.isInteger(dados.peso_nascimento)) {
    msg.textContent = "Peso ao nascer: informe em gramas, sem vírgula (ex.: 3200)";
    return;
  }
  if ([dados.ig_semanas, dados.ig_dias].some((n) => n !== null && !Number.isInteger(n))) {
    msg.textContent = "Idade gestacional: informe semanas e dias em números inteiros";
    return;
  }
  try {
    if (editandoId === null) await api("/puericultura/api/criancas/create", "POST", dados);
    else await api("/puericultura/api/criancas/update", "POST", { id: editandoId, ...dados });
    fecharModais();
    showMsg(editandoId === null ? "Criança cadastrada." : "Cadastro atualizado.", true);
    await load();
  } catch (e) {
    msg.textContent = e.message;
  }
}


// ----------------------------- Wiring -----------------------------

for (const aba of ABAS) $("aba-" + aba).addEventListener("click", () => mostrarAba(aba));
$("novo-btn").addEventListener("click", () => abrirForm(null));
$("form").addEventListener("submit", salvar);
for (const id of ["f-nascimento", "f-ig", "f-ig-dias", "f-ultima"]) $(id).addEventListener("input", previa);

$("busca").addEventListener("input", (e) => { filtros.busca = e.target.value; render(); });
// Choosing an ACS is the ACS's view: who is late and needs an active search. The button still turns it off.
$("filtro-acs").addEventListener("change", (e) => {
  filtros.acs = e.target.value;
  if (filtros.acs) filtros.atrasadas = true;
  render();
});
$("filtro-atrasadas").addEventListener("click", () => { filtros.atrasadas = !filtros.atrasadas; render(); });

for (const overlay of document.querySelectorAll(".modal-overlay")) {
  overlay.addEventListener("click", (e) => { if (e.target === overlay || e.target.closest("[data-fechar]")) fecharModais(); });
}
document.addEventListener("keydown", (e) => { if (e.key === "Escape") fecharModais(); });

load();
