# Dioxus Plumb

Draw blocks with words in it, connect those blocks with arrows.

## Getting Started

Your new bare-bones project includes minimal organization with a single `main.rs` file and a few assets.

```
project/
├─ assets/ # Any assets that are used by the app should be placed here
├─ src/
│  ├─ main.rs # main.rs is the entry point to your application and currently contains all components for the app
├─ Cargo.toml # The Cargo.toml file defines the dependencies and feature flags for your project
```

### Tailwind
1. Install npm: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
2. Install the Tailwind CSS CLI: https://tailwindcss.com/docs/installation
3. Run the following command in the root of the project to start the Tailwind CSS compiler:

```bash
npx tailwindcss -i ./tailwind.css -o ./assets/tailwind.css --watch
```

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```

## TODO 

1. **Edge Labels Positioning**:
   - Improve the edge label positioning by calculating a perpendicular offset from the midpoint of the curved path for better readability
   - Add a small background to the labels to make them more readable when crossing other elements

2. **Zoom and Pan Controls**:
   - Add zoom and pan controls to navigate large graphs
   - Implement mouse wheel zoom and drag-to-pan behavior

3. **Node and Edge Styling**:
   - Support additional node attributes like shape, color, fill
   - Add edge styling options (dashed, dotted, thickness)
   - Support highlighting of connected nodes on hover

4. **Performance Improvements**:
   - Implement virtualization for large graphs (only render visible nodes)
   - Optimize arrow calculations by caching positions when possible

5. **Export Options**:
   - Add functionality to export the rendered graph as SVG or PNG

6. **Interactive Features**:
   - Add node dragging capability to rearrange layouts
   - Implement collapsible subgraphs
   - Add context menu on nodes and edges for additional actions

7. **Layout Algorithm**:
   - Implement automatic layout algorithms (force-directed, hierarchical)
   - Add option to toggle between different layout algorithms

8. **Accessibility**:
   - Improve keyboard navigation
   - Add ARIA attributes for screen readers
   - Support high contrast mode
