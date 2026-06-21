const PERIODOS = ["Diurno", "Noturno", "Cinderela", "24 noturno"];
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
  const body = document.getElementById("plantoes-body");
  body.innerHTML = "";
  plantoes.forEach((p, i) => body.appendChild(rowView(p, i + 1)));
  renderDashboard();
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
  tr.className = p.recebido ? "recebido" : "pendente";
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
    <td><input type="text" data-k="local" value="${esc(p.local)}"></td>
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
