import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Plus, Search, Scan, Bot } from "lucide-react";
import { useAgents, useScanAgents } from "@/hooks/useAgents";
import { Spinner } from "@/components/Spinner/Spinner";
import { EmptyState } from "@/components/EmptyState/EmptyState";
import AgentCard from "@/components/AgentCard";
import AddAgentModal from "@/components/AddAgentModal";
import styles from "./Agents.module.css";

export default function Agents() {
  const navigate = useNavigate();
  const { data: agents, isLoading } = useAgents();
  const scanMutation = useScanAgents();
  const [showAddModal, setShowAddModal] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [filterType, setFilterType] = useState<string>("all");

  const filteredAgents = agents?.filter((a) => {
    const matchesSearch = a.agent.name.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesType = filterType === "all" || a.agent.agent_type === filterType;
    return matchesSearch && matchesType;
  });

  const agentTypes = ["all", ...new Set(agents?.map((a) => a.agent.agent_type) ?? [])];

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h1 className={styles.title}>Agents</h1>
        <div className={styles.actions}>
          <button
            className="btn btn-secondary"
            onClick={() => scanMutation.mutate()}
            disabled={scanMutation.isPending}
          >
            <Scan size={16} />
            {scanMutation.isPending ? "Scanning..." : "Scan for Agents"}
          </button>
          <button className="btn btn-primary" onClick={() => setShowAddModal(true)}>
            <Plus size={16} />
            Add Agent
          </button>
        </div>
      </div>

      <div className={styles.filters}>
        <div className={styles.searchWrapper}>
          <Search size={16} className={styles.searchIcon} />
          <input
            type="text"
            placeholder="Search agents..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className={styles.searchInput}
          />
        </div>
        <select
          value={filterType}
          onChange={(e) => setFilterType(e.target.value)}
          className={styles.filterSelect}
        >
          {agentTypes.map((type) => (
            <option key={type} value={type}>
              {type === "all" ? "All Types" : type}
            </option>
          ))}
        </select>
      </div>

      {isLoading ? (
        <div className={styles.loading}><Spinner /> Loading agents...</div>
      ) : filteredAgents && filteredAgents.length > 0 ? (
        <div className={styles.grid}>
          {filteredAgents.map((agent) => (
            <AgentCard
              key={agent.agent.id}
              agent={agent}
              onClick={() => navigate(`/agents/${agent.agent.id}`)}
            />
          ))}
        </div>
      ) : (
        <EmptyState
          icon={Bot}
          title="No agents installed"
          description="Add an agent to get started"
          action={{ label: "Add Agent", onClick: () => setShowAddModal(true) }}
        />
      )}

      {showAddModal && <AddAgentModal onClose={() => setShowAddModal(false)} />}
    </div>
  );
}
