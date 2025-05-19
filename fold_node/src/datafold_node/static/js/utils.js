// Initialize utils module immediately
(function(global) {
    console.log('Initializing utils module...');

    const utils = {
        displayResult: function(data, isError = false) {
            const resultsDiv = document.getElementById('results');
            if (!resultsDiv) {
                console.error('Results div not found');
                return;
            }
            
            // Format the data
            const formattedData = typeof data === 'string' ? data : JSON.stringify(data, null, 2);
            
            // Add appropriate styling
            resultsDiv.className = '';
            resultsDiv.innerHTML = formattedData;
            
            // Scroll to results
            resultsDiv.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
            
            // Update node status indicator
            const nodeStatus = document.getElementById('nodeStatus');
            if (nodeStatus) {
                nodeStatus.className = isError ? 'status error' : 'status success';
                const statusIcon = document.getElementById('statusIcon');
                if (statusIcon && window.icons) {
                    statusIcon.innerHTML = isError ? icons.x() : icons.check();
                }
            }
        },

        isValidJSON: function(str) {
            try {
                JSON.parse(str);
                return true;
            } catch (e) {
                return false;
            }
        },

        showLoading: function(element, message = 'Loading...') {
            if (!element) {
                console.error('Cannot show loading: element is null');
                return;
            }
            
            element.innerHTML = `
                <div class="status info">
                    <span class="loading"></span>
                    <span>${message}</span>
                </div>
            `;
        },

        handleApiError: function(error) {
            console.error('API Error:', error);
            
            // Update node status indicator to show error
            const nodeStatus = document.getElementById('nodeStatus');
            if (nodeStatus) {
                nodeStatus.className = 'status error';
                const statusIcon = document.getElementById('statusIcon');
                if (statusIcon && window.icons) {
                    statusIcon.innerHTML = icons.x();
                }
            }
            
            return `Error: ${error.message || 'Unknown error occurred'}`;
        },

        apiRequest: async function(url, options = {}) {
            try {
                const response = await fetch(url, options);
                
                // Handle 404 errors specifically
                if (response.status === 404) {
                    console.warn(`API endpoint not found: ${url}`);
                    throw new Error(`API endpoint not found: ${url}`);
                }
                
                // Try to parse JSON response
                let data;
                try {
                    data = await response.json();
                } catch (parseError) {
                    console.error('Failed to parse API response as JSON:', parseError);
                    throw new Error('Invalid API response format');
                }
                
                // Log the response data for debugging
                console.log('API Response:', data);
                
                if (data.error) {
                    throw new Error(data.error);
                }
                
                // If data field is missing and this isn't an error response,
                // wrap the response in a data field
                if (!data.hasOwnProperty('data') && !data.error) {
                    return { data };
                }
                
                return data;
            } catch (error) {
                throw new Error(this.handleApiError(error));
            }
        },

        switchTab: function(tabName) {
            console.log('Switching to tab:', tabName);
            
            // Update tab buttons
            const tabButtons = document.querySelectorAll('.tab-button');
            if (tabButtons.length === 0) {
                console.error('No tab buttons found');
                return;
            }
            
            tabButtons.forEach(button => {
                button.classList.remove('active');
            });
            
            const activeButton = document.querySelector(`.tab-button[data-tab="${tabName}"]`);
            if (activeButton) {
                activeButton.classList.add('active');
            } else {
                console.error(`Tab button for ${tabName} not found`);
            }

            // Update tab content containers
            const tabContainers = document.querySelectorAll('.tab-content-container');
            if (tabContainers.length === 0) {
                console.error('No tab content containers found');
                return;
            }
            
            tabContainers.forEach(container => {
                container.classList.remove('active');
            });
            
            // Map tab names to container IDs
            const containerMap = {
                'schemas': 'schemaTabsContainer',
                'schema': 'schemaTabsContainer',
                'query': 'operationsTabsContainer',
                'mutation': 'operationsTabsContainer',
                'samples': 'samplesTabContainer',
                'transforms': 'transformsTabContainer',
                'network': 'networkTabContainer'
            };
            
            const containerId = containerMap[tabName];
            if (!containerId) {
                console.error(`No container mapping for tab: ${tabName}`);
                return;
            }
            
            const activeContainer = document.getElementById(containerId);
            if (activeContainer) {
                activeContainer.classList.add('active');
                
                // Also update inner tab content if it exists
                const innerContent = activeContainer.querySelector(`#${tabName}Tab`);
                if (innerContent) {
                    activeContainer.querySelectorAll('.tab-content').forEach(content => {
                        content.classList.remove('active');
                    });
                    innerContent.classList.add('active');
                }
                
                // Refresh data when switching to certain tabs
                if (tabName === 'schemas' && window.schemaModule) {
                    schemaModule.loadSchemaList();
                } else if (tabName === 'network' && window.networkModule) {
                    networkModule.getNetworkStatus();
                }
            } else {
                console.error(`Container ${containerId} not found`);
            }
        },

        toggleSchema: function(element) {
            if (element.classList.contains('collapsed')) {
                element.classList.remove('collapsed');
                element.classList.add('expanded');
            } else {
                element.classList.remove('expanded');
                element.classList.add('collapsed');
            }
        },

        showNotification: function(message, type = 'info', duration = 3000) {
            // Create notification element if it doesn't exist
            let notificationContainer = document.getElementById('notification-container');
            
            if (!notificationContainer) {
                notificationContainer = document.createElement('div');
                notificationContainer.id = 'notification-container';
                notificationContainer.style.position = 'fixed';
                notificationContainer.style.top = '20px';
                notificationContainer.style.right = '20px';
                notificationContainer.style.zIndex = '1000';
                document.body.appendChild(notificationContainer);
            }
            
            // Create notification
            const notification = document.createElement('div');
            notification.className = `alert alert-${type} fade-in`;
            notification.style.marginBottom = '10px';
            
            // Add icon based on type
            let icon = '';
            if (window.icons) {
                switch (type) {
                    case 'success': icon = icons.check(); break;
                    case 'error': icon = icons.x(); break;
                    case 'warning': icon = icons.warning(); break;
                    case 'info': icon = icons.info(); break;
                }
            }
            
            notification.innerHTML = `${icon} ${message}`;
            
            // Add to container
            notificationContainer.appendChild(notification);
            
            // Remove after duration
            setTimeout(() => {
                notification.style.opacity = '0';
                setTimeout(() => {
                    notificationContainer.removeChild(notification);
                }, 300);
            }, duration);
        },

        loadHtmlIntoContainer: function(url, containerId, callback) {
            console.log(`Loading HTML from ${url} into ${containerId}`);
            
            return new Promise((resolve, reject) => {
                fetch(url)
                    .then(response => {
                        if (!response.ok) {
                            throw new Error(`Failed to load ${url}: ${response.status} ${response.statusText}`);
                        }
                        return response.text();
                    })
                    .then(html => {
                        const container = document.getElementById(containerId);
                        if (!container) {
                            throw new Error(`Container ${containerId} not found`);
                        }

                        container.innerHTML = html;

                        // Execute any inline scripts within the loaded HTML
                        const scripts = container.querySelectorAll('script');
                        const scriptPromises = Array.from(scripts).map(script => {
                            return new Promise((scriptResolve) => {
                                const newScript = document.createElement('script');
                                
                                if (script.src) {
                                    newScript.src = script.src;
                                    newScript.onload = scriptResolve;
                                    newScript.onerror = () => {
                                        console.error(`Failed to load script: ${script.src}`);
                                        scriptResolve(); // Resolve anyway to not block other scripts
                                    };
                                } else {
                                    newScript.textContent = script.textContent;
                                    scriptResolve();
                                }
                                
                                document.body.appendChild(newScript);
                                if (!script.src) {
                                    document.body.removeChild(newScript);
                                }
                            });
                        });

                        // Wait for all scripts to load
                        Promise.all(scriptPromises)
                            .then(() => {
                                if (typeof callback === 'function') {
                                    callback(container);
                                }
                                resolve(container);
                            })
                            .catch(error => {
                                console.error('Error executing scripts:', error);
                                reject(error);
                            });
                    })
                    .catch(error => {
                        console.error('Failed to load HTML component:', error);
                        reject(error);
                    });
            });
        }
    };

    // Make utils globally available
    global.utils = utils;
    console.log('Utils module initialized and available globally');

})(typeof window !== 'undefined' ? window : global);
