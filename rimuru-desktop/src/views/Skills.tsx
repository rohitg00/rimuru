import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Search, Package, Store, Sparkles, RefreshCw, Tag } from "lucide-react";
import { Spinner } from "@/components/Spinner/Spinner";
import { EmptyState } from "@/components/EmptyState/EmptyState";
import {
  useInstalledSkills,
  useSearchSkills,
  useSkillRecommendations,
} from "@/hooks/useSkills";
import { SKILLKIT_AGENTS, Skill } from "@/lib/tauri";
import SkillCard from "@/components/SkillCard";
import SkillInstallModal from "@/components/SkillInstallModal";
import styles from "./Skills.module.css";
import clsx from "clsx";

type Tab = "installed" | "marketplace" | "recommendations";

export default function Skills() {
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<Tab>("installed");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedAgent, setSelectedAgent] = useState<string>("all");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [skillToInstall, setSkillToInstall] = useState<Skill | null>(null);

  const { data: installedSkills, isLoading: installedLoading, refetch: refetchInstalled } =
    useInstalledSkills(selectedAgent === "all" ? undefined : selectedAgent);

  const { data: searchResults, isLoading: searchLoading, refetch: refetchSearch } = useSearchSkills(
    {
      query: searchQuery || undefined,
      agent: selectedAgent === "all" ? undefined : selectedAgent,
      tags: selectedTags.length > 0 ? selectedTags : undefined,
      limit: 50,
    },
    activeTab === "marketplace"
  );

  const { data: recommendations, isLoading: recommendationsLoading, refetch: refetchRecommendations } =
    useSkillRecommendations();

  const handleTabChange = (tab: Tab) => {
    setActiveTab(tab);
    setSearchQuery("");
  };

  const handleRefresh = () => {
    if (activeTab === "installed") {
      refetchInstalled();
    } else if (activeTab === "marketplace") {
      refetchSearch();
    } else {
      refetchRecommendations();
    }
  };

  const filteredInstalled = installedSkills?.filter((skill) => {
    if (!searchQuery) return true;
    return (
      skill.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      skill.description.toLowerCase().includes(searchQuery.toLowerCase())
    );
  });

  const popularTags = ["ai", "testing", "database", "api", "frontend", "devops", "security"];

  const handleTagToggle = (tag: string) => {
    setSelectedTags((prev) =>
      prev.includes(tag) ? prev.filter((t) => t !== tag) : [...prev, tag]
    );
  };

  const isLoading =
    (activeTab === "installed" && installedLoading) ||
    (activeTab === "marketplace" && searchLoading) ||
    (activeTab === "recommendations" && recommendationsLoading);

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h1 className={styles.title}>Skills</h1>
        <button className="btn btn-secondary" onClick={handleRefresh}>
          <RefreshCw size={16} className={isLoading ? styles.spinning : ""} />
          Refresh
        </button>
      </div>

      <div className={styles.tabs}>
        <button
          className={clsx(styles.tab, activeTab === "installed" && styles.active)}
          onClick={() => handleTabChange("installed")}
        >
          <Package size={16} />
          Installed
          {installedSkills && (
            <span className={styles.tabCount}>{installedSkills.length}</span>
          )}
        </button>
        <button
          className={clsx(styles.tab, activeTab === "marketplace" && styles.active)}
          onClick={() => handleTabChange("marketplace")}
        >
          <Store size={16} />
          Marketplace
        </button>
        <button
          className={clsx(styles.tab, activeTab === "recommendations" && styles.active)}
          onClick={() => handleTabChange("recommendations")}
        >
          <Sparkles size={16} />
          Recommendations
          {recommendations && recommendations.length > 0 && (
            <span className={styles.tabCount}>{recommendations.length}</span>
          )}
        </button>
      </div>

      <div className={styles.filters}>
        <div className={styles.searchWrapper}>
          <Search size={16} className={styles.searchIcon} />
          <input
            type="text"
            placeholder={
              activeTab === "installed"
                ? "Search installed skills..."
                : activeTab === "marketplace"
                ? "Search marketplace..."
                : "Search recommendations..."
            }
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className={styles.searchInput}
          />
        </div>

        <select
          value={selectedAgent}
          onChange={(e) => setSelectedAgent(e.target.value)}
          className={styles.filterSelect}
        >
          <option value="all">All Agents</option>
          {SKILLKIT_AGENTS.map((agent) => (
            <option key={agent.value} value={agent.value}>
              {agent.icon} {agent.label}
            </option>
          ))}
        </select>
      </div>

      {activeTab === "marketplace" && (
        <div className={styles.tagFilters}>
          <Tag size={14} className={styles.tagIcon} />
          {popularTags.map((tag) => (
            <button
              key={tag}
              className={clsx(
                styles.tagFilter,
                selectedTags.includes(tag) && styles.tagSelected
              )}
              onClick={() => handleTagToggle(tag)}
            >
              {tag}
            </button>
          ))}
        </div>
      )}

      <div className={styles.content}>
        {isLoading ? (
          <div className={styles.loading}>
            <Spinner />
            <span>Loading skills...</span>
          </div>
        ) : activeTab === "installed" ? (
          filteredInstalled && filteredInstalled.length > 0 ? (
            <div className={styles.grid}>
              {filteredInstalled.map((skill) => (
                <SkillCard
                  key={skill.id}
                  skill={skill}
                  onClick={() => navigate(`/skills/${skill.id}`)}
                />
              ))}
            </div>
          ) : (
            <EmptyState
              icon={Sparkles}
              title="No skills found"
              description="Install skills to extend your agents"
              action={{ label: "Browse Marketplace", onClick: () => setActiveTab("marketplace") }}
            />
          )
        ) : activeTab === "marketplace" ? (
          searchResults && searchResults.skills.length > 0 ? (
            <>
              <div className={styles.resultsInfo}>
                Found {searchResults.total} skills
              </div>
              <div className={styles.grid}>
                {searchResults.skills.map((skill) => (
                  <SkillCard
                    key={skill.id}
                    skill={skill}
                    onClick={() => navigate(`/skills/${skill.id}`)}
                    onInstall={() => setSkillToInstall(skill)}
                    showInstallButton
                  />
                ))}
              </div>
            </>
          ) : (
            <EmptyState
              icon={Store}
              title="No skills found"
              description="Try adjusting your search or filters"
            />
          )
        ) : recommendations && recommendations.length > 0 ? (
          <div className={styles.grid}>
            {recommendations.map((rec) => (
              <SkillCard
                key={rec.skill.id}
                skill={rec.skill}
                onClick={() => navigate(`/skills/${rec.skill.id}`)}
                onInstall={() => setSkillToInstall(rec.skill)}
                showInstallButton
                confidence={rec.confidence}
                reason={rec.reason}
              />
            ))}
          </div>
        ) : (
          <EmptyState
            icon={Sparkles}
            title="No recommendations yet"
            description="Use Rimuru more to get personalized skill recommendations"
          />
        )}
      </div>

      {skillToInstall && (
        <SkillInstallModal
          skill={skillToInstall}
          onClose={() => setSkillToInstall(null)}
          onSuccess={() => {
            refetchInstalled();
            refetchSearch();
          }}
        />
      )}
    </div>
  );
}
