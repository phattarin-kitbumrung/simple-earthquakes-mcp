from langchain_ollama import OllamaLLM
from langgraph.graph import END, StateGraph
from langchain_core.messages import HumanMessage, AIMessage
from typing import TypedDict, Annotated, Sequence, Dict
import requests
import json

# 1. Connect to local Ollama
llm = OllamaLLM(model="qwen2.5:7b")

# 2. Define MCP tool
def call_mcp(query: str) -> str:
    try:
        response = requests.get(
            "http://localhost:3000/earthquakes",
            params={"query": query},
            headers={"Accept": "application/json"}
        )
        response.raise_for_status()
        data = response.json()
        return data
    except Exception as e:
        return f"Error calling MCP: {e}"

# 3. Define state structure
class AgentState(TypedDict):
    messages: Annotated[Sequence[HumanMessage | AIMessage], "The conversation history"]
    tools_output: Dict
    next: str

# 4. Define agent nodes
def agent_node(state: AgentState) -> AgentState:
    """Process user input and decide what to do next."""
    messages = state["messages"]
    # Get the last user message
    user_message = messages[-1].content if messages and isinstance(messages[-1], HumanMessage) else ""
    
    # Simple logic to determine if we need to use tool
    if "earthquake" in user_message.lower() or "quake" in user_message.lower():
        return {"messages": messages, "next": "use_tool"}
    else:
        response = llm.invoke(user_message)
        messages.append(AIMessage(content=response))
        return {"messages": messages, "next": END}

def use_tool(state: AgentState) -> AgentState:
    """Use the MCP tool to get earthquake data."""
    messages = state["messages"]
    user_message = messages[-1].content if messages and isinstance(messages[-1], HumanMessage) else ""
    
    # Call the MCP tool
    tool_result = call_mcp(user_message)

    # Store tool output
    state["tools_output"] = {"ContextRetriever": tool_result}
    
    return {"messages": messages, "tools_output": state["tools_output"], "next": "process_tool_output"}

def process_tool_output(state: AgentState) -> AgentState:
    """Process the tool output and form a response."""
    messages = state["messages"]
    tool_output = state["tools_output"].get("ContextRetriever", {})
    
    user_message = messages[-1].content
    context = f"I found this information: {json.dumps(tool_output)}"
    prompt = f"The user asked: \"{user_message}\"\n\nBased on this data: {context}\n\nPlease answer the user's question."
    response = llm.invoke(prompt)
    
    messages.append(AIMessage(content=response))

    return {"messages": messages, "tools_output": tool_output, "next": END}

# 5. Create the graph
workflow = StateGraph(AgentState)

# Add nodes
workflow.add_node("agent", agent_node)
workflow.add_node("use_tool", use_tool)
workflow.add_node("process_tool_output", process_tool_output)

# Add edges
workflow.add_edge("agent", "use_tool")
workflow.add_edge("use_tool", "process_tool_output")
workflow.set_entry_point("agent")

# 6. Compile the graph
graph = workflow.compile()

# 7. Ask something
messages = [HumanMessage(content="How many earthquakes have occurred in Thailand?")]
result = graph.invoke({"messages": messages})

print("\n🧠 Answer:\n", result["messages"][-1].content)
