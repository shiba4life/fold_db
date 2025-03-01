# DataFold Node Web UI

This directory contains the web UI for the DataFold Node. The UI has been restructured to be more maintainable and modular.

## Directory Structure

```
static/
├── css/                  # CSS stylesheets
│   └── styles.css        # Main stylesheet
├── js/                   # JavaScript modules
│   ├── app.js            # Main application initialization
│   ├── utils.js          # Utility functions
│   ├── schema.js         # Schema-related operations
│   ├── operations.js     # Query and mutation operations
│   └── network.js        # Network-related operations
├── components/           # HTML components
│   ├── schema-tab.html   # Schema tab components
│   ├── operations-tab.html # Query and mutation tab components
│   └── network-tab.html  # Network tab component
└── index.html            # Main HTML file
```

## Architecture

The web UI follows a modular architecture with clear separation of concerns:

1. **HTML Components**: Each major UI section is separated into its own HTML component file, which is loaded dynamically.
2. **CSS**: All styles are centralized in a single CSS file for easier maintenance.
3. **JavaScript Modules**: Code is organized into logical modules:
   - `utils.js`: Common utility functions
   - `schema.js`: Schema-related operations
   - `operations.js`: Query and mutation operations
   - `network.js`: Network-related operations
   - `app.js`: Application initialization and event handling

## Features

- **Modular Design**: Each component can be developed and tested independently.
- **Dynamic Loading**: Components are loaded dynamically to improve initial page load time.
- **Improved Error Handling**: Consistent error handling across all operations.
- **Loading Indicators**: Visual feedback during asynchronous operations.
- **Responsive Design**: UI adapts to different screen sizes.
- **Consistent Styling**: Unified styling across all components.

## How to Extend

### Adding a New Tab

1. Create a new HTML component file in the `components/` directory.
2. Add a new tab button in `index.html`.
3. Create a new JavaScript module in the `js/` directory if needed.
4. Update `app.js` to handle the new tab's event listeners.

### Modifying Existing Components

1. Locate the relevant component file in the `components/` directory.
2. Make your changes to the HTML structure.
3. Update the corresponding JavaScript module if needed.
4. Update the CSS if new styles are required.

## Best Practices

1. **Keep Components Small**: Each component should focus on a single responsibility.
2. **Use Utility Functions**: Common operations should be added to `utils.js`.
3. **Consistent Error Handling**: Use the `utils.displayResult()` function for displaying results and errors.
4. **Document Your Code**: Add JSDoc comments to functions and components.
5. **Responsive Design**: Ensure components work well on different screen sizes.
