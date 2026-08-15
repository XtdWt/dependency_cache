import matplotlib.pyplot as plt
import networkx as nx

from .dependency_cache import DependencyCacheBase


def plot_dependency_graph(obj, **kwargs) -> None:
    if not isinstance(obj, DependencyCacheBase):
        raise TypeError("provided object must inherit from DependencyCacheBase")
    graph_data = obj.get_dependency_graph()
    g = nx.DiGraph()
    for child_node, parents in graph_data.items():
        if isinstance(child_node, tuple):
            func_name, args = child_node
            child_node = f"{func_name}({''.join([f'{var}={val}' for (var, val) in args])})"
        for parent_node in parents:
            if isinstance(parent_node, tuple):
                func_name, args = parent_node
                parent_node = f"{func_name}({''.join([f'{var}={val}' for (var, val) in args])})"
            g.add_edge(parent_node, child_node)

    for layer_idx, nodes in enumerate(nx.topological_generations(g)):
        for node in nodes:
            g.nodes[node]["layer"] = layer_idx

    pos = nx.multipartite_layout(g, subset_key="layer", align="horizontal")
    pos = {node: (coords[0], -coords[1]) for node, coords in pos.items()}

    defaults = {
        "with_labels": True,
        "node_size": 3600,
        "node_color": "#4361ee",
        "font_color": "white",
        "font_weight": "bold",
        "edge_color": "gray",
        "width": 1,
        "arrows": True,
    }

    draw_options = defaults | kwargs

    nx.draw(g, pos=pos, **draw_options)
    plt.show()
