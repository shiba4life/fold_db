/**
 * Transform management for the DataFold Node UI
 */

async function loadTransforms() {
    const container = document.getElementById('transformsList');
    if (!container) return;
    utils.showLoading(container, 'Loading transforms...');
    try {
        const resp = await utils.apiRequest('/api/transforms');
        const map = resp.data || {};
        container.innerHTML = '';
        Object.entries(map).forEach(([id, t]) => {
            const div = document.createElement('div');
            div.className = 'transform-item';
            const pre = document.createElement('pre');
            pre.textContent = JSON.stringify(t, null, 2);
            const btn = document.createElement('button');
            btn.className = 'btn btn-sm btn-primary mt-2';
            btn.innerHTML = `${window.icons ? icons.play() + ' ' : ''}Run`;
            btn.addEventListener('click', () => runTransform(id));
            div.appendChild(document.createTextNode(id));
            div.appendChild(btn);
            div.appendChild(pre);
            container.appendChild(div);
        });
    } catch (e) {
        container.innerHTML = 'Failed to load transforms';
    }
}

async function runTransform(id) {
    try {
        const resp = await utils.apiRequest(`/api/transform/${id}/run`, {method: 'POST'});
        utils.displayResult(resp.data);
    } catch (e) {
        utils.displayResult(e.message, true);
    }
}

window.transformsModule = { loadTransforms, runTransform };
