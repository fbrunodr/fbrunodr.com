const PERIODOS = ["Diurno", "Noturno", "Cinderela", "24hrs", "24hrs invertido"];
const TIPOS = ["PA", "MFC"];

let plantoes = [];

function brl(n) {
  return (n || 0).toLocaleString("pt-BR", { style: "currency", currency: "BRL" });
}

// Returns an ISO yyyy-mm-dd one month after the given ISO date.
// Built from date parts (not Date parsing) to avoid timezone shifts.
function addOneMonth(iso) {
  const parts = iso.split("-").map(Number);
  if (parts.length !== 3) return "";
  const dt = new Date(parts[0], parts[1] - 1, parts[2]);
  dt.setMonth(dt.getMonth() + 1);
  const yy = dt.getFullYear();
  const mm = String(dt.getMonth() + 1).padStart(2, "0");
  const dd = String(dt.getDate()).padStart(2, "0");
  return `${yy}-${mm}-${dd}`;
}

// dd/mm/aaaa from an ISO yyyy-mm-dd (avoids timezone shifts)
function fmtDate(iso) {
  if (!iso) return "";
  const parts = iso.split("-");
  if (parts.length !== 3) return iso;
  return `${parts[2]}/${parts[1]}/${parts[0]}`;
}

// True when an ISO yyyy-mm-dd date is strictly after today (shift hasn't
// happened yet). String comparison is safe for the yyyy-mm-dd format.
function isFuture(iso) {
  if (!iso) return false;
  const now = new Date();
  const today = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
  return iso > today;
}

function showMsg(text, ok) {
  const el = document.getElementById("msg");
  el.textContent = text;
  el.className = "msg " + (ok ? "ok" : "err");
  if (text) setTimeout(() => { el.textContent = ""; el.className = "msg"; }, 4000);
}

function esc(s) {
  const d = document.createElement("div");
  d.textContent = s == null ? "" : s;
  return d.innerHTML;
}

async function api(path, method, body) {
  const opts = { method, headers: {} };
  if (body !== undefined) {
    opts.headers["Content-Type"] = "application/json";
    opts.body = JSON.stringify(body);
  }
  const res = await fetch(path, opts);
  if (res.status === 401) {
    window.location.href = "/lucasodon";
    throw new Error("nao autorizado");
  }
  if (!res.ok) throw new Error(await res.text());
  return res;
}

async function load() {
  try {
    const res = await api("/lucasodon/api/list", "GET");
    plantoes = await res.json();
    render();
  } catch (e) {
    showMsg("Erro ao carregar: " + e.message, false);
  }
}

function renderTotals() {
  let recebido = 0, pendente = 0;
  for (const p of plantoes) {
    if (p.recebido) recebido += p.valor;
    else pendente += p.valor;
  }
  document.getElementById("totais").innerHTML = `
    <div class="card"><div class="lbl">Total recebido</div><div class="val recebido">${brl(recebido)}</div></div>
    <div class="card"><div class="lbl">A receber</div><div class="val pendente">${brl(pendente)}</div></div>
    <div class="card"><div class="lbl">Plant&otilde;es</div><div class="val">${plantoes.length}</div></div>
  `;
}

function render() {
  renderTotals();
  renderLocaisDatalist();
  const body = document.getElementById("plantoes-body");
  body.innerHTML = "";
  plantoes.forEach((p, i) => body.appendChild(rowView(p, i + 1)));
  renderDashboard();
}

// Autocomplete suggestions for the Local field: distinct locals used in the
// last 12 months. The user can still type a brand-new place.
function renderLocaisDatalist() {
  const now = new Date();
  const c = new Date(now.getFullYear(), now.getMonth() - 12, now.getDate());
  const cutoff = `${c.getFullYear()}-${String(c.getMonth() + 1).padStart(2, "0")}-${String(c.getDate()).padStart(2, "0")}`;

  const locais = new Set();
  for (const p of plantoes) {
    if (p.local && p.data >= cutoff) locais.add(p.local);
  }

  document.getElementById("locais-list").innerHTML =
    [...locais].sort().map(l => `<option value="${esc(l)}"></option>`).join("");
}

const MESES = ["Janeiro", "Fevereiro", "Março", "Abril", "Maio", "Junho",
  "Julho", "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro"];

// Bucket a plantao by its payment month. Prefer the expected-payment date
// (ISO, sortable); fall back to the shift date. Returns {key, label}.
function paymentMonth(p) {
  const iso = p.previsao_pagamento || p.data;
  const parts = (iso || "").split("-");
  if (parts.length !== 3) return { key: "9999-99", label: "Sem data" };
  return {
    key: `${parts[0]}-${parts[1]}`,
    label: `${MESES[Number(parts[1]) - 1]} ${parts[0]}`,
  };
}

function pct(part, total) {
  return total > 0 ? (part / total) * 100 : 0;
}

function renderDashboard() {
  const groups = {};
  for (const p of plantoes) {
    const { key, label } = paymentMonth(p);
    if (!groups[key]) groups[key] = { key, label, total: 0, recebido: 0, pendente: 0, count: 0 };
    const g = groups[key];
    g.total += p.valor;
    g.count += 1;
    if (p.recebido) g.recebido += p.valor;
    else g.pendente += p.valor;
  }

  const months = Object.values(groups).sort((a, b) => a.key.localeCompare(b.key));
  const grandTotal = months.reduce((s, m) => s + m.total, 0);
  const grandReceb = months.reduce((s, m) => s + m.recebido, 0);
  const grandPend = months.reduce((s, m) => s + m.pendente, 0);

  const el = document.getElementById("view-dashboard");

  if (months.length === 0) {
    el.innerHTML = `<p class="dash-empty">Nenhum plantão lançado ainda.</p>`;
    return;
  }

  const summary = `
    <div class="dash-summary">
      <div class="card"><div class="lbl">Total previsto</div><div class="val">${brl(grandTotal)}</div></div>
      <div class="card"><div class="lbl">Recebido</div><div class="val recebido">${brl(grandReceb)}</div></div>
      <div class="card"><div class="lbl">A receber</div><div class="val pendente">${brl(grandPend)}</div></div>
    </div>`;

  const cards = months.map(m => `
    <div class="dash-month">
      <div class="dash-month-head">
        <span class="mlabel">${m.label}</span>
        <span class="mcount">${m.count} plantã${m.count === 1 ? "o" : "os"}</span>
      </div>
      <div class="dash-bar" title="Recebido ${brl(m.recebido)} de ${brl(m.total)}">
        <div class="bar-recebido" style="width:${pct(m.recebido, m.total)}%"></div>
        <div class="bar-pendente" style="width:${pct(m.pendente, m.total)}%"></div>
      </div>
      <div class="dash-figs">
        <div><span class="lbl">Previsto</span><span class="v">${brl(m.total)}</span></div>
        <div><span class="lbl">Recebido</span><span class="v recebido">${brl(m.recebido)}</span></div>
        <div><span class="lbl">A receber</span><span class="v pendente">${brl(m.pendente)}</span></div>
      </div>
    </div>`).join("");

  el.innerHTML = summary + `<div class="dash-grid">${cards}</div>`;
}

function setView(view) {
  const isDash = view === "dashboard";
  document.getElementById("view-table").style.display = isDash ? "none" : "";
  document.getElementById("view-dashboard").style.display = isDash ? "" : "none";
  document.getElementById("tab-table").classList.toggle("active", !isDash);
  document.getElementById("tab-dash").classList.toggle("active", isDash);
}

function rowView(p, num) {
  const tr = document.createElement("tr");
  tr.className = isFuture(p.data) ? "futuro" : (p.recebido ? "recebido" : "pendente");
  tr.innerHTML = `
    <td>${num}</td>
    <td>${fmtDate(p.data)}</td>
    <td>${esc(p.local)}</td>
    <td>${esc(p.mfc_pa)}</td>
    <td>${p.duracao_h}</td>
    <td>${esc(p.periodo)}</td>
    <td>${brl(p.valor)}</td>
    <td>${brl(p.valor_hora)}</td>
    <td>${fmtDate(p.previsao_pagamento)}</td>
    <td>${p.recebido ? "Sim" : "N&atilde;o"}</td>
    <td>${fmtDate(p.mes_ano_pagamento)}</td>
    <td>${esc(p.observacoes)}</td>
  `;
  const actions = document.createElement("td");
  actions.className = "actions";
  const editBtn = document.createElement("button");
  editBtn.className = "edit";
  editBtn.textContent = "Editar";
  editBtn.onclick = () => tr.replaceWith(rowEdit(p, num));
  const delBtn = document.createElement("button");
  delBtn.className = "del";
  delBtn.textContent = "Excluir";
  delBtn.onclick = () => remove(p.id);
  actions.appendChild(editBtn);
  actions.appendChild(delBtn);
  tr.appendChild(actions);
  return tr;
}

function selectHtml(options, selected) {
  return options.map(o => `<option${o === selected ? " selected" : ""}>${o}</option>`).join("");
}

function rowEdit(p, num) {
  const tr = document.createElement("tr");
  tr.innerHTML = `
    <td>${num}</td>
    <td><input type="date" data-k="data" value="${esc(p.data)}"></td>
    <td><input type="text" data-k="local" list="locais-list" value="${esc(p.local)}"></td>
    <td><select data-k="mfc_pa">${selectHtml(TIPOS, p.mfc_pa)}</select></td>
    <td><input type="number" step="0.5" data-k="duracao_h" value="${p.duracao_h}"></td>
    <td><select data-k="periodo">${selectHtml(PERIODOS, p.periodo)}</select></td>
    <td><input type="number" step="0.01" data-k="valor" value="${p.valor}"></td>
    <td>-</td>
    <td><input type="date" data-k="previsao_pagamento" value="${esc(p.previsao_pagamento)}"></td>
    <td><input type="checkbox" data-k="recebido"${p.recebido ? " checked" : ""}></td>
    <td><input type="date" data-k="mes_ano_pagamento" value="${esc(p.mes_ano_pagamento)}"></td>
    <td><input type="text" data-k="observacoes" value="${esc(p.observacoes)}"></td>
  `;
  // Auto-fill expected payment one month after the shift date.
  const dataInput = tr.querySelector('[data-k="data"]');
  const prevInput = tr.querySelector('[data-k="previsao_pagamento"]');
  dataInput.addEventListener("change", () => {
    if (dataInput.value) prevInput.value = addOneMonth(dataInput.value);
  });

  const actions = document.createElement("td");
  actions.className = "actions";
  const saveBtn = document.createElement("button");
  saveBtn.className = "save";
  saveBtn.textContent = "Salvar";
  saveBtn.onclick = () => save(p.id, tr);
  const cancelBtn = document.createElement("button");
  cancelBtn.className = "edit";
  cancelBtn.textContent = "Cancelar";
  cancelBtn.onclick = () => tr.replaceWith(rowView(p, num));
  actions.appendChild(saveBtn);
  actions.appendChild(cancelBtn);
  tr.appendChild(actions);
  return tr;
}

function collect(scope) {
  const get = k => scope.querySelector(`[data-k="${k}"]`);
  const g = id => document.getElementById(id);
  const read = (el) => {
    if (!el) return null;
    if (el.type === "checkbox") return el.checked;
    return el.value;
  };
  // scope is a row (edit) or null (top form using f- ids)
  const pick = scope
    ? k => read(get(k))
    : k => read(g("f-" + k));

  return {
    data: pick("data"),
    local: (pick("local") || "").trim(),
    mfc_pa: pick("mfc_pa"),
    duracao_h: parseFloat(pick("duracao_h")) || 0,
    periodo: pick("periodo"),
    valor: parseFloat(pick("valor")) || 0,
    previsao_pagamento: pick("previsao_pagamento") || null,
    recebido: !!pick("recebido"),
    mes_ano_pagamento: (pick("mes_ano_pagamento") || "").trim() || null,
    observacoes: (pick("observacoes") || "").trim() || null,
  };
}

function validate(d) {
  if (!d.data) return "Informe a data.";
  if (!d.local) return "Informe o local.";
  if (d.duracao_h <= 0) return "Duração deve ser maior que zero.";
  return null;
}

function resetForm() {
  ["f-data", "f-local", "f-duracao_h", "f-valor", "f-previsao_pagamento", "f-mes_ano_pagamento", "f-observacoes"]
    .forEach(id => { document.getElementById(id).value = ""; });
  document.getElementById("f-mfc_pa").selectedIndex = 0;
  document.getElementById("f-periodo").selectedIndex = 0;
  document.getElementById("f-recebido").checked = false;
}

function openForm() {
  document.getElementById("form-modal").classList.add("open");
  document.getElementById("f-data").focus();
}

function closeForm() {
  document.getElementById("form-modal").classList.remove("open");
}

async function add() {
  const d = collect(null);
  const err = validate(d);
  if (err) { showMsg(err, false); return; }
  try {
    await api("/lucasodon/api/create", "POST", d);
    resetForm();
    closeForm();
    showMsg("Plantão adicionado.", true);
    await load();
  } catch (e) {
    showMsg("Erro ao adicionar: " + e.message, false);
  }
}

async function save(id, tr) {
  const d = collect(tr);
  const err = validate(d);
  if (err) { showMsg(err, false); return; }
  try {
    await api("/lucasodon/api/update", "POST", Object.assign({ id }, d));
    showMsg("Plantão atualizado.", true);
    await load();
  } catch (e) {
    showMsg("Erro ao salvar: " + e.message, false);
  }
}

async function remove(id) {
  if (!confirm("Excluir este plantão?")) return;
  try {
    await api("/lucasodon/api/delete", "POST", { id });
    showMsg("Plantão excluído.", true);
    await load();
  } catch (e) {
    showMsg("Erro ao excluir: " + e.message, false);
  }
}

// Auto-fill the expected payment date to one month after the shift date.
document.getElementById("f-data").addEventListener("change", () => {
  const v = document.getElementById("f-data").value;
  if (v) document.getElementById("f-previsao_pagamento").value = addOneMonth(v);
});

document.getElementById("tab-table").addEventListener("click", () => setView("table"));
document.getElementById("tab-dash").addEventListener("click", () => setView("dashboard"));

document.getElementById("add-btn").addEventListener("click", add);
document.getElementById("open-form-btn").addEventListener("click", openForm);
document.getElementById("close-form-btn").addEventListener("click", closeForm);
document.getElementById("cancel-form-btn").addEventListener("click", closeForm);
// Close when clicking the dark backdrop (but not the modal box itself)
document.getElementById("form-modal").addEventListener("click", (e) => {
  if (e.target.id === "form-modal") closeForm();
});
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") closeForm();
});
load();


/* =========================================================================
   Despesas module
   ========================================================================= */

let categorias = [], catById = {}, despesas = [], recorrentes = [];
let despesasLoaded = false, editingDespesaId = null, editingRecorrenteId = null;

function showDespMsg(text, ok) {
  const el = document.getElementById("despesas-msg");
  el.textContent = text;
  el.className = "msg " + (ok ? "ok" : "err");
  if (text) setTimeout(() => { el.textContent = ""; el.className = "msg"; }, 4000);
}

// ---- main tab switching ----
function setMainTab(tab) {
  const isDesp = tab === "despesas";
  document.getElementById("section-plantoes").style.display = isDesp ? "none" : "";
  document.getElementById("section-despesas").style.display = isDesp ? "" : "none";
  document.getElementById("maintab-plantoes").classList.toggle("active", !isDesp);
  document.getElementById("maintab-despesas").classList.toggle("active", isDesp);
  if (isDesp && !despesasLoaded) { despesasLoaded = true; initDespesas(); }
}

async function initDespesas() {
  const mes = document.getElementById("d-mes");
  if (!mes.value) {
    const now = new Date();
    mes.value = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
  }
  await loadCategorias();
  await loadRecorrentes();
  await loadDespesas();
}

// ---- categorias ----
async function loadCategorias() {
  try {
    const res = await api("/lucasodon/api/categorias", "GET");
    categorias = await res.json();
    catById = {};
    for (const c of categorias) catById[c.id] = c;
    fillCategoriaSelects();
    renderCatList();
  } catch (e) { showDespMsg("Erro ao carregar categorias: " + e.message, false); }
}

function fillCategoriaSelects() {
  const opts = categorias.map(c => `<option value="${c.id}">${esc(c.nome)}</option>`).join("");
  const keep = (id, v) => { const el = document.getElementById(id); el.innerHTML = v; };
  keep("dx-categoria", opts);
  keep("rx-categoria", opts);
  document.getElementById("d-filtro-categoria").innerHTML = `<option value="">Todas</option>` + opts;
}

function renderCatList() {
  const ul = document.getElementById("cat-list");
  ul.innerHTML = "";
  for (const c of categorias) {
    const li = document.createElement("li");
    const span = document.createElement("span");
    span.className = "cat-item-nome";
    span.textContent = c.nome;
    const ed = document.createElement("button");
    ed.className = "edit"; ed.textContent = "Renomear"; ed.onclick = () => renomearCategoria(c);
    const del = document.createElement("button");
    del.className = "del"; del.textContent = "Excluir"; del.onclick = () => excluirCategoria(c);
    li.append(span, ed, del);
    ul.appendChild(li);
  }
}

async function addCategoria() {
  const input = document.getElementById("cat-novo");
  const nome = input.value.trim();
  if (!nome) return;
  try {
    await api("/lucasodon/api/categorias/create", "POST", { nome });
    input.value = "";
    await loadCategorias();
  } catch (e) { showDespMsg("Erro: " + e.message, false); }
}

async function renomearCategoria(c) {
  const nome = prompt("Novo nome da categoria:", c.nome);
  if (!nome || !nome.trim()) return;
  try {
    await api("/lucasodon/api/categorias/update", "POST", { id: c.id, nome: nome.trim() });
    await loadCategorias();
    await loadDespesas();
  } catch (e) { showDespMsg("Erro: " + e.message, false); }
}

async function excluirCategoria(c) {
  if (!confirm(`Excluir a categoria "${c.nome}"? As despesas dela vão para "Outros".`)) return;
  try {
    await api("/lucasodon/api/categorias/delete", "POST", { id: c.id });
    await loadCategorias();
    await loadDespesas();
  } catch (e) { showDespMsg("Erro: " + e.message, false); }
}

// ---- despesas: load + render ----
function despesaRange() {
  const tipo = document.getElementById("d-periodo-tipo").value;
  const mes = document.getElementById("d-mes").value;
  const [y, m] = mes.split("-").map(Number);
  if (tipo === "ano") return { inicio: `${y}-01-01`, fim: `${y}-12-31` };
  const last = new Date(y, m, 0).getDate();
  const mm = String(m).padStart(2, "0");
  return { inicio: `${y}-${mm}-01`, fim: `${y}-${mm}-${String(last).padStart(2, "0")}` };
}

async function loadDespesas() {
  const { inicio, fim } = despesaRange();
  const qs = new URLSearchParams({ inicio, fim });
  const cat = document.getElementById("d-filtro-categoria").value;
  const tipo = document.getElementById("d-filtro-tipo").value;
  if (cat) qs.set("categoria_id", cat);
  if (tipo && tipo !== "todos") qs.set("tipo", tipo);
  try {
    const res = await api(`/lucasodon/api/despesas?${qs.toString()}`, "GET");
    despesas = await res.json();
    renderDespesas();
  } catch (e) { showDespMsg("Erro ao carregar: " + e.message, false); }
}

function renderDespesas() {
  let total = 0, pessoal = 0, profissional = 0;
  const porCat = {};
  for (const d of despesas) {
    total += d.valor;
    if (d.tipo === "profissional") profissional += d.valor; else pessoal += d.valor;
    const key = d.categoria_id || 0;
    porCat[key] = (porCat[key] || 0) + d.valor;
  }

  const cards = `
    <div class="card"><div class="lbl">Total do período</div><div class="val">${brl(total)}</div></div>
    <div class="card"><div class="lbl">Pessoal</div><div class="val">${brl(pessoal)}</div></div>
    <div class="card"><div class="lbl">Profissional</div><div class="val profissional">${brl(profissional)}</div></div>`;

  const linhas = Object.entries(porCat).sort((a, b) => b[1] - a[1]).map(([id, v]) => {
    const nome = id === "0" ? "Sem categoria" : (catById[id] ? catById[id].nome : "—");
    const p = total > 0 ? (v / total) * 100 : 0;
    return `<div class="cat-row"><span class="cat-nome">${esc(nome)}</span>
      <div class="cat-bar"><div style="width:${p}%"></div></div>
      <span class="cat-val">${brl(v)}</span></div>`;
  }).join("");

  document.getElementById("despesas-totais").innerHTML = cards +
    `<div class="cat-breakdown">${linhas || '<span class="dash-empty">Sem despesas no período.</span>'}</div>`;

  const body = document.getElementById("despesas-body");
  body.innerHTML = "";
  if (!despesas.length) {
    body.innerHTML = `<tr><td colspan="7" class="dash-empty">Sem despesas no período.</td></tr>`;
    return;
  }
  despesas.forEach(d => body.appendChild(despesaRow(d)));
}

function despesaRow(d) {
  const tr = document.createElement("tr");
  if (d.status === "previsto") tr.className = "previsto";
  const catNome = d.categoria_id && catById[d.categoria_id] ? catById[d.categoria_id].nome : "—";
  tr.innerHTML = `
    <td>${fmtDate(d.data)}</td>
    <td>${esc(d.descricao)}${d.recorrente_id ? ' <span class="badge-rec">fixa</span>' : ""}</td>
    <td>${esc(catNome)}</td>
    <td>${d.tipo === "profissional" ? "Profissional" : "Pessoal"}</td>
    <td>${brl(d.valor)}</td>
    <td><span class="pill ${d.status}">${d.status === "pago" ? "Pago" : "Previsto"}</span></td>`;
  const acts = document.createElement("td");
  acts.className = "actions";
  if (d.status === "previsto") {
    const pay = document.createElement("button");
    pay.className = "save"; pay.textContent = "Pagar"; pay.onclick = () => pagarDespesa(d);
    acts.appendChild(pay);
  }
  const ed = document.createElement("button");
  ed.className = "edit"; ed.textContent = "Editar"; ed.onclick = () => openDespesa(d);
  const del = document.createElement("button");
  del.className = "del"; del.textContent = "Excluir"; del.onclick = () => removeDespesa(d);
  acts.append(ed, del);
  tr.appendChild(acts);
  return tr;
}

async function pagarDespesa(d) {
  try {
    await api("/lucasodon/api/despesas/update", "POST", {
      id: d.id, valor: d.valor, data: d.data, descricao: d.descricao,
      categoria_id: d.categoria_id, tipo: d.tipo, status: "pago", observacoes: d.observacoes,
    });
    showDespMsg("Despesa marcada como paga.", true);
    await loadDespesas();
  } catch (e) { showDespMsg("Erro: " + e.message, false); }
}

async function removeDespesa(d) {
  const msg = d.recorrente_id
    ? "Excluir esta ocorrência da despesa fixa? Ela não será gerada de novo."
    : "Excluir esta despesa?";
  if (!confirm(msg)) return;
  try {
    await api("/lucasodon/api/despesas/delete", "POST", { id: d.id });
    showDespMsg("Despesa excluída.", true);
    await loadDespesas();
  } catch (e) { showDespMsg("Erro: " + e.message, false); }
}

// ---- despesa modal ----
function openDespesa(d) {
  editingDespesaId = d ? d.id : null;
  document.getElementById("despesa-modal-title").textContent = d ? "Editar despesa" : "Nova despesa";
  document.getElementById("dx-data").value = d ? d.data : "";
  document.getElementById("dx-valor").value = d ? d.valor : "";
  document.getElementById("dx-descricao").value = d ? (d.descricao || "") : "";
  document.getElementById("dx-categoria").value = d && d.categoria_id ? String(d.categoria_id) : "";
  document.getElementById("dx-tipo").value = d ? d.tipo : "pessoal";
  document.getElementById("dx-status").value = d ? d.status : "pago";
  document.getElementById("dx-observacoes").value = d ? (d.observacoes || "") : "";
  document.getElementById("despesa-modal").classList.add("open");
}

function closeDespesa() { document.getElementById("despesa-modal").classList.remove("open"); }

async function saveDespesa() {
  const data = document.getElementById("dx-data").value;
  const valor = parseFloat(document.getElementById("dx-valor").value) || 0;
  if (!data) { showDespMsg("Informe a data.", false); return; }
  if (valor <= 0) { showDespMsg("Informe um valor.", false); return; }
  const catVal = document.getElementById("dx-categoria").value;
  const body = {
    valor, data,
    descricao: document.getElementById("dx-descricao").value.trim() || null,
    categoria_id: catVal ? Number(catVal) : null,
    tipo: document.getElementById("dx-tipo").value,
    status: document.getElementById("dx-status").value,
    observacoes: document.getElementById("dx-observacoes").value.trim() || null,
  };
  try {
    if (editingDespesaId) await api("/lucasodon/api/despesas/update", "POST", Object.assign({ id: editingDespesaId }, body));
    else await api("/lucasodon/api/despesas/create", "POST", body);
    closeDespesa();
    showDespMsg("Despesa salva.", true);
    await loadDespesas();
  } catch (e) { showDespMsg("Erro: " + e.message, false); }
}

// ---- recorrentes ----
async function loadRecorrentes() {
  try {
    const res = await api("/lucasodon/api/recorrentes", "GET");
    recorrentes = await res.json();
    renderRecorrentes();
  } catch (e) { showDespMsg("Erro ao carregar recorrentes: " + e.message, false); }
}

function renderRecorrentes() {
  const body = document.getElementById("recorrentes-body");
  body.innerHTML = "";
  if (!recorrentes.length) {
    body.innerHTML = `<tr><td colspan="9" class="dash-empty">Nenhuma despesa fixa cadastrada.</td></tr>`;
    return;
  }
  for (const r of recorrentes) body.appendChild(recorrenteRow(r));
}

function recorrenteRow(r) {
  const tr = document.createElement("tr");
  if (!r.ativo) tr.className = "inativo";
  const cat = r.categoria_id && catById[r.categoria_id] ? catById[r.categoria_id].nome : "—";
  const venc = r.periodicidade === "anual"
    ? `dia ${r.dia_vencimento} de ${MESES[(r.mes_vencimento || 1) - 1]}`
    : `dia ${r.dia_vencimento}`;
  const vig = `${fmtDate(r.data_inicio)} – ${r.data_fim ? fmtDate(r.data_fim) : "..."}`;
  tr.innerHTML = `
    <td>${esc(r.descricao)}</td>
    <td>${esc(cat)}</td>
    <td>${r.tipo === "profissional" ? "Profissional" : "Pessoal"}</td>
    <td>${brl(r.valor)}</td>
    <td>${r.periodicidade === "anual" ? "Anual" : "Mensal"}</td>
    <td>${venc}</td>
    <td>${vig}</td>
    <td>${r.ativo ? "Sim" : "Não"}</td>`;
  const acts = document.createElement("td");
  acts.className = "actions";
  const ed = document.createElement("button");
  ed.className = "edit"; ed.textContent = "Editar"; ed.onclick = () => openRecorrente(r);
  const del = document.createElement("button");
  del.className = "del"; del.textContent = "Excluir"; del.onclick = () => removeRecorrente(r);
  acts.append(ed, del);
  tr.appendChild(acts);
  return tr;
}

function toggleMesField() {
  document.getElementById("rx-mes-field").style.display =
    document.getElementById("rx-periodicidade").value === "anual" ? "" : "none";
}

function openRecorrente(r) {
  editingRecorrenteId = r ? r.id : null;
  document.getElementById("recorrente-modal-title").textContent = r ? "Editar recorrente" : "Nova recorrente";
  document.getElementById("rx-descricao").value = r ? r.descricao : "";
  document.getElementById("rx-valor").value = r ? r.valor : "";
  document.getElementById("rx-categoria").value = r && r.categoria_id ? String(r.categoria_id) : "";
  document.getElementById("rx-tipo").value = r ? r.tipo : "pessoal";
  document.getElementById("rx-periodicidade").value = r ? r.periodicidade : "mensal";
  document.getElementById("rx-dia").value = r ? r.dia_vencimento : "";
  document.getElementById("rx-mes").value = r && r.mes_vencimento ? String(r.mes_vencimento) : "1";
  document.getElementById("rx-inicio").value = r ? r.data_inicio : "";
  document.getElementById("rx-fim").value = r && r.data_fim ? r.data_fim : "";
  document.getElementById("rx-ativo").checked = r ? r.ativo : true;
  toggleMesField();
  document.getElementById("recorrente-modal").classList.add("open");
}

function closeRecorrente() { document.getElementById("recorrente-modal").classList.remove("open"); }

async function saveRecorrente() {
  const descricao = document.getElementById("rx-descricao").value.trim();
  const valor = parseFloat(document.getElementById("rx-valor").value) || 0;
  const dia = parseInt(document.getElementById("rx-dia").value, 10) || 0;
  const inicio = document.getElementById("rx-inicio").value;
  const periodicidade = document.getElementById("rx-periodicidade").value;
  if (!descricao) { showDespMsg("Informe a descrição.", false); return; }
  if (valor <= 0) { showDespMsg("Informe um valor.", false); return; }
  if (dia < 1 || dia > 31) { showDespMsg("Dia de vencimento inválido (1–31).", false); return; }
  if (!inicio) { showDespMsg("Informe a data de início.", false); return; }
  const catVal = document.getElementById("rx-categoria").value;
  const body = {
    descricao, valor,
    categoria_id: catVal ? Number(catVal) : null,
    tipo: document.getElementById("rx-tipo").value,
    periodicidade,
    dia_vencimento: dia,
    mes_vencimento: periodicidade === "anual" ? Number(document.getElementById("rx-mes").value) : null,
    data_inicio: inicio,
    data_fim: document.getElementById("rx-fim").value || null,
    ativo: document.getElementById("rx-ativo").checked,
  };
  try {
    if (editingRecorrenteId) await api("/lucasodon/api/recorrentes/update", "POST", Object.assign({ id: editingRecorrenteId }, body));
    else await api("/lucasodon/api/recorrentes/create", "POST", body);
    closeRecorrente();
    showDespMsg("Despesa fixa salva.", true);
    await loadRecorrentes();
    await loadDespesas();
  } catch (e) { showDespMsg("Erro: " + e.message, false); }
}

async function removeRecorrente(r) {
  if (!confirm("Excluir esta despesa fixa? Ocorrências futuras ainda não pagas serão removidas.")) return;
  try {
    await api("/lucasodon/api/recorrentes/delete", "POST", { id: r.id });
    showDespMsg("Despesa fixa excluída.", true);
    await loadRecorrentes();
    await loadDespesas();
  } catch (e) { showDespMsg("Erro: " + e.message, false); }
}

// ---- wiring ----
function bindModalClose(modalId, ...closers) {
  const modal = document.getElementById(modalId);
  closers.forEach(id => document.getElementById(id).addEventListener("click", () => modal.classList.remove("open")));
  modal.addEventListener("click", (e) => { if (e.target.id === modalId) modal.classList.remove("open"); });
}

document.getElementById("maintab-plantoes").addEventListener("click", () => setMainTab("plantoes"));
document.getElementById("maintab-despesas").addEventListener("click", () => setMainTab("despesas"));

["d-periodo-tipo", "d-mes", "d-filtro-categoria", "d-filtro-tipo"].forEach(id =>
  document.getElementById(id).addEventListener("change", loadDespesas));

document.getElementById("open-despesa-btn").addEventListener("click", () => openDespesa(null));
document.getElementById("save-despesa-btn").addEventListener("click", saveDespesa);
bindModalClose("despesa-modal", "close-despesa-btn", "cancel-despesa-btn");

document.getElementById("open-recorrente-btn").addEventListener("click", () => openRecorrente(null));
document.getElementById("save-recorrente-btn").addEventListener("click", saveRecorrente);
document.getElementById("rx-periodicidade").addEventListener("change", toggleMesField);
bindModalClose("recorrente-modal", "close-recorrente-btn", "cancel-recorrente-btn");

document.getElementById("open-categorias-btn").addEventListener("click", () => document.getElementById("categorias-modal").classList.add("open"));
document.getElementById("cat-add-btn").addEventListener("click", addCategoria);
bindModalClose("categorias-modal", "close-categorias-btn");
