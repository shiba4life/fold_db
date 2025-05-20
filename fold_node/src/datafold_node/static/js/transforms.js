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
            
            // Transform ID and Run button
            const header = document.createElement('div');
            header.className = 'transform-header';
            header.appendChild(document.createTextNode(id));
            const btn = document.createElement('button');
            btn.className = 'btn btn-sm btn-primary';
            btn.innerHTML = `${window.icons ? icons.play() + ' ' : ''}Run`;
            btn.addEventListener('click', () => runTransform(id));
            header.appendChild(btn);
            div.appendChild(header);

            // Output field display
            if (t.output) {
                const outputSection = document.createElement('div');
                outputSection.className = 'transform-output';
                const outputLabel = document.createElement('div');
                outputLabel.className = 'transform-label';
                outputLabel.textContent = 'Output:';
                const outputValue = document.createElement('pre');
                outputValue.className = 'output-value';
                outputValue.textContent = JSON.stringify(t.output, null, 2);
                outputSection.appendChild(outputLabel);
                outputSection.appendChild(outputValue);
                div.appendChild(outputSection);
            }

            // Full transform details
            const detailsLabel = document.createElement('div');
            detailsLabel.className = 'transform-label';
            detailsLabel.textContent = 'Transform Details:';
            const pre = document.createElement('pre');
            pre.className = 'transform-details';
            pre.textContent = JSON.stringify(t, null, 2);
            div.appendChild(detailsLabel);
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
