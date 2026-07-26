import matplotlib.pyplot as plt
import networkx as nx

from .dependency_cache import DependencyCacheBase


def plot_dependency_graph(obj, **kwargs) -> None:
    if not isinstance(obj, DependencyCacheBase):
        raise TypeError("provided object must inherit from DependencyCacheBase")
    graph_data = obj.current_graph()
    g = nx.DiGraph()
    for child_node, parents in graph_data.items():
        for parent_node in parents:
            g.add_edge(parent_node, child_node)

    for layer_idx, nodes in enumerate(nx.topological_generations(g)):
        for node in nodes:
            g.nodes[node]["layer"] = layer_idx

    pos = nx.multipartite_layout(g, subset_key="layer", align="horizontal")
    pos = {node: (coords[0], -coords[1]) for node, coords in pos.items()}

    defaults = {
        "with_labels": True,
        "node_size": 1200,
        "node_color": "#4361ee",
        "font_color": "white",
        "font_weight": "bold",
        "edge_color": "gray",
        "width": 2,
        "arrows": True,
    }

    draw_options = defaults | kwargs

    nx.draw(g, pos=pos, **draw_options)
    plt.show()
