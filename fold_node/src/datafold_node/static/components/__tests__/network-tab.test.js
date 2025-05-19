const { fireEvent } = require('@testing-library/dom');
require('@testing-library/jest-dom');

// Mock the icons module
window.icons = {
    play: () => '<svg>play</svg>',
    stop: () => '<svg>stop</svg>',
    search: () => '<svg>search</svg>',
    folder: () => '<svg>folder</svg>',
    gear: () => '<svg>gear</svg>',
    network: () => '<svg>network</svg>'
};

// Mock the network module
const mockRenderNetworkStatus = jest.fn().mockImplementation((status) => {
    const container = document.getElementById('networkStatus');
    if (!container) return;
    
    container.innerHTML = '';
    Object.entries(status).forEach(([key, value]) => {
        const card = document.createElement('div');
        card.className = `status-card ${key === 'last_error' ? 'error' : ''}`;
        card.textContent = value;
        container.appendChild(card);
    });
});

window.networkModule = {
    initNetwork: jest.fn(),
    connectToNode: jest.fn(),
    renderNetworkStatus: mockRenderNetworkStatus
};

// Mock utils module
window.utils = {
    displayResult: jest.fn()
};

describe('Network Tab Component', () => {
    let container;

    beforeEach(() => {
        // Set up DOM
        const html = require('../network-tab.html');
        document.body.innerHTML = html;
        container = document.getElementById('networkTab');
        
        // Execute the script content from the HTML
        const scriptContent = html.match(/<script>([\s\S]*?)<\/script>/)[1];
        eval(scriptContent);
        
        // Reset all mock functions
        jest.clearAllMocks();
    });

    afterEach(() => {
        // Clear mocks
        jest.clearAllMocks();
    });

    test('renders network tab component', () => {
        expect(container).toBeInTheDocument();
        expect(container.querySelector('#networkStatus')).toBeInTheDocument();
        expect(container.querySelector('#networkConfigForm')).toBeInTheDocument();
    });

    test('initializes network with correct configuration', () => {
        // Set form values
        const listenAddress = container.querySelector('#listenAddress');
        const discoveryPort = container.querySelector('#discoveryPort');
        const maxConnections = container.querySelector('#maxConnections');
        const connectionTimeout = container.querySelector('#connectionTimeout');
        const announcementInterval = container.querySelector('#announcementInterval');
        const enableDiscovery = container.querySelector('#enableDiscovery');

        // Set form values
        fireEvent.change(listenAddress, { target: { value: '127.0.0.1:9000' } });
        fireEvent.change(discoveryPort, { target: { value: '9001' } });
        fireEvent.change(maxConnections, { target: { value: '20' } });
        fireEvent.change(connectionTimeout, { target: { value: '45' } });
        fireEvent.change(announcementInterval, { target: { value: '90' } });
        
        // Ensure discovery is enabled by setting checked property
        enableDiscovery.checked = true;

        // Click initialize button
        const initButton = container.querySelector('#initNetworkBtn');
        fireEvent.click(initButton);

        // Verify network initialization called with correct config
        expect(networkModule.initNetwork).toHaveBeenCalledWith({
            listen_address: '127.0.0.1:9000',
            discovery_port: 9001,
            max_connections: 20,
            connection_timeout_secs: 45,
            announcement_interval_secs: 90,
            enable_discovery: true
        });
    });

    test('handles node connection', () => {
        const nodeIdInput = container.querySelector('#nodeId');
        const connectButton = container.querySelector('#connectToNodeBtn');

        // Test empty node ID
        fireEvent.click(connectButton);
        expect(utils.displayResult).toHaveBeenCalledWith('Please enter a node ID', true);
        expect(networkModule.connectToNode).not.toHaveBeenCalled();

        // Test valid node ID
        fireEvent.change(nodeIdInput, { target: { value: 'test-node-123' } });
        fireEvent.click(connectButton);
        expect(networkModule.connectToNode).toHaveBeenCalledWith('test-node-123');
    });

    test('renders network status correctly', () => {
        const status = {
            running: true,
            connected_nodes: 5,
            discovery_enabled: true,
            listen_address: '127.0.0.1:8000',
            last_error: null
        };

        networkModule.renderNetworkStatus(status);

        const statusContainer = container.querySelector('#networkStatus');
        const statusCards = statusContainer.querySelectorAll('.status-card');

        expect(statusCards).toHaveLength(4); // 4 status cards without error
        expect(statusCards[0]).toHaveTextContent('Running');
        expect(statusCards[1]).toHaveTextContent('5');
        expect(statusCards[2]).toHaveTextContent('Enabled');
        expect(statusCards[3]).toHaveTextContent('127.0.0.1:8000');
    });

    test('renders network status with error', () => {
        const status = {
            running: false,
            connected_nodes: 0,
            discovery_enabled: true,
            listen_address: '127.0.0.1:8000',
            last_error: 'Connection failed'
        };

        networkModule.renderNetworkStatus(status);

        const statusContainer = container.querySelector('#networkStatus');
        const statusCards = statusContainer.querySelectorAll('.status-card');
        const errorCard = statusContainer.querySelector('.status-card.error');

        expect(statusCards).toHaveLength(5); // 5 status cards with error
        expect(errorCard).toHaveTextContent('Connection failed');
    });
});