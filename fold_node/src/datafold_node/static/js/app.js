/**
 * Main application file for the DataFold Node UI
 */

class App {
    constructor() {
        this.componentsLoaded = false;
        this.initialized = false;
    }

    async init() {
        if (this.initialized) return;
        
        try {
            // Initialize icons first
            this.initIcons();
            
            // Load all components
            await this.loadComponents();
            
            // Set up event listeners
            this.setupEventListeners();
            
            // Load initial data
            await this.loadInitialData();
            
            this.initialized = true;
            console.log('DataFold Node UI initialized');
        } catch (error) {
            console.error('Failed to initialize app:', error);
        }
    }

    initIcons() {
        if (!window.icons) {
            console.error('Icons module not loaded');
            return;
        }

        const iconMappings = {
            'logoIcon': icons.database,
            'schemasTabIcon': icons.folder,
            'schemaTabIcon': icons.schema,
            'queryTabIcon': icons.search,
            'mutationTabIcon': icons.code,
            'samplesTabIcon': icons.library,
            'transformsTabIcon': icons.gear,
            'networkTabIcon': icons.network,
            'statusIcon': icons.check
        };

        Object.entries(iconMappings).forEach(([id, iconFn]) => {
            const element = document.getElementById(id);
            if (element) {
                element.innerHTML = iconFn();
            }
        });
    }

    async loadComponents() {
        if (this.componentsLoaded) return;

        const components = [
            { url: 'components/schema-tab.html', id: 'schemaTabsContainer' },
            { url: 'components/operations-tab.html', id: 'operationsTabsContainer' },
            { url: 'components/transforms-tab.html', id: 'transformsTabContainer' },
            { url: 'components/network-tab.html', id: 'networkTabContainer' },
            { url: 'components/samples-tab.html', id: 'samplesTabContainer' }
        ];

        try {
            await Promise.all(components.map(comp =>
                utils.loadHtmlIntoContainer(comp.url, comp.id)
            ));
            this.componentsLoaded = true;
        } catch (error) {
            console.error('Failed to load components:', error);
            throw error;
        }
    }

    setupEventListeners() {
        // Tab switching
        document.querySelectorAll('.tab-button').forEach(button => {
            const tabName = button.getAttribute('data-tab');
            if (tabName) {
                button.addEventListener('click', () => utils.switchTab(tabName));
            }
        });

        // Schema tab
        const loadSchemaBtn = document.getElementById('loadSchemaBtn');
        if (loadSchemaBtn) {
            loadSchemaBtn.addEventListener('click', () => schemaModule.loadSchema());
        }

        const loadExampleSchemaBtn = document.getElementById('loadExampleSchemaBtn');
        if (loadExampleSchemaBtn) {
            loadExampleSchemaBtn.addEventListener('click', () => schemaModule.loadExampleSchema());
        }

        // Operations
        const runQueryBtn = document.getElementById('runQueryBtn');
        if (runQueryBtn) {
            runQueryBtn.addEventListener('click', () => operationsModule.executeOperation('query'));
        }

        const runMutationBtn = document.getElementById('runMutationBtn');
        if (runMutationBtn) {
            runMutationBtn.addEventListener('click', () => operationsModule.executeOperation('mutation'));
        }

        // Network
        ['startNetwork', 'stopNetwork', 'discoverNodes', 'listNodes'].forEach(action => {
            const btn = document.getElementById(`${action}Btn`);
            if (btn && networkModule[action]) {
                btn.addEventListener('click', () => networkModule[action]());
            }
        });
    }

    async loadInitialData() {
        try {
            // Initialize transforms
            if (window.transformsModule) {
                const refreshIcon = document.getElementById('refreshTransformsIcon');
                if (window.icons && refreshIcon) {
                    refreshIcon.innerHTML = icons.refresh();
                }
                const refreshBtn = document.getElementById('refreshTransformsBtn');
                if (refreshBtn) {
                    refreshBtn.addEventListener('click', () => transformsModule.loadTransforms());
                }
                await transformsModule.loadTransforms();
            }

            // Initialize samples
            if (window.samplesModule?.initSamplesTab) {
                await samplesModule.initSamplesTab();
            }

            // Load schema list
            await schemaModule.loadSchemaList();

            // Load network status
            if (document.getElementById('networkStatus')) {
                await networkModule.getNetworkStatus();
            }
        } catch (error) {
            console.error('Failed to load initial data:', error);
        }
    }
}

// Create and export app instance
window.app = new App();

// Initialize when everything is loaded
window.addEventListener('load', () => {
    window.app.init().catch(error => {
        console.error('Failed to initialize app:', error);
    });
});
