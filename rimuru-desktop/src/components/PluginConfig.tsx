import { useState, useEffect } from "react";
import { X, Save, RotateCcw, ExternalLink, Info } from "lucide-react";
import { usePluginConfig, useConfigurePlugin } from "@/hooks/usePlugins";
import { Plugin } from "@/lib/tauri";
import styles from "./PluginConfig.module.css";

interface PluginConfigProps {
  plugin: Plugin;
  onClose: () => void;
}

interface SchemaProperty {
  type: string;
  title?: string;
  description?: string;
  default?: unknown;
  enum?: string[];
  additionalProperties?: { type: string };
}

interface Schema {
  type: string;
  properties?: Record<string, SchemaProperty>;
  required?: string[];
}

export default function PluginConfig({ plugin, onClose }: PluginConfigProps) {
  const { data: config, isLoading, refetch } = usePluginConfig(plugin.id);
  const configureMutation = useConfigurePlugin();
  const [formValues, setFormValues] = useState<Record<string, unknown>>({});
  const [hasChanges, setHasChanges] = useState(false);

  useEffect(() => {
    if (config?.settings) {
      setFormValues(config.settings as Record<string, unknown>);
    }
  }, [config]);

  const schema = config?.schema as Schema | undefined;

  const handleChange = (key: string, value: unknown) => {
    setFormValues((prev) => ({
      ...prev,
      [key]: value,
    }));
    setHasChanges(true);
  };

  const handleSaveAll = async () => {
    for (const [key, value] of Object.entries(formValues)) {
      try {
        await configureMutation.mutateAsync({
          plugin_id: plugin.id,
          key,
          value,
        });
      } catch (error) {
        alert(`Failed to save ${key}: ${error}`);
        return;
      }
    }
    setHasChanges(false);
    refetch();
  };

  const handleReset = () => {
    if (config?.settings) {
      setFormValues(config.settings as Record<string, unknown>);
      setHasChanges(false);
    }
  };

  const renderField = (key: string, property: SchemaProperty) => {
    const value = formValues[key];
    const isRequired = schema?.required?.includes(key);

    switch (property.type) {
      case "boolean":
        return (
          <label className={styles.toggle}>
            <input
              type="checkbox"
              checked={Boolean(value ?? property.default)}
              onChange={(e) => handleChange(key, e.target.checked)}
            />
            <span className={styles.toggleSlider} />
          </label>
        );

      case "integer":
      case "number":
        return (
          <input
            type="number"
            value={String(value ?? property.default ?? "")}
            onChange={(e) => handleChange(key, Number(e.target.value))}
            className={styles.input}
            required={isRequired}
          />
        );

      case "string":
        if (property.enum) {
          return (
            <select
              value={String(value ?? property.default ?? "")}
              onChange={(e) => handleChange(key, e.target.value)}
              className={styles.select}
              required={isRequired}
            >
              <option value="">Select...</option>
              {property.enum.map((opt) => (
                <option key={opt} value={opt}>
                  {opt}
                </option>
              ))}
            </select>
          );
        }
        return (
          <input
            type={key.toLowerCase().includes("url") ? "url" : "text"}
            value={String(value ?? property.default ?? "")}
            onChange={(e) => handleChange(key, e.target.value)}
            className={styles.input}
            placeholder={property.default ? String(property.default) : undefined}
            required={isRequired}
          />
        );

      case "object":
        return (
          <textarea
            value={typeof value === "object" ? JSON.stringify(value, null, 2) : "{}"}
            onChange={(e) => {
              try {
                handleChange(key, JSON.parse(e.target.value));
              } catch {
              }
            }}
            className={styles.textarea}
            rows={4}
            placeholder='{"key": "value"}'
          />
        );

      default:
        return (
          <input
            type="text"
            value={String(value ?? "")}
            onChange={(e) => handleChange(key, e.target.value)}
            className={styles.input}
          />
        );
    }
  };

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.panel} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <div className={styles.headerInfo}>
            <h2 className={styles.title}>{plugin.name}</h2>
            <span className={styles.version}>v{plugin.version}</span>
          </div>
          <button className={styles.closeBtn} onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <div className={styles.meta}>
          <p className={styles.description}>{plugin.description}</p>
          <div className={styles.metaDetails}>
            <span>Author: {plugin.author}</span>
            {plugin.license && <span>License: {plugin.license}</span>}
            {plugin.repository && (
              <a
                href={plugin.repository}
                target="_blank"
                rel="noopener noreferrer"
                className={styles.repoLink}
              >
                <ExternalLink size={14} />
                Repository
              </a>
            )}
          </div>
        </div>

        <div className={styles.content}>
          {isLoading ? (
            <div className={styles.loading}>Loading configuration...</div>
          ) : schema?.properties ? (
            <div className={styles.form}>
              {Object.entries(schema.properties).map(([key, property]) => (
                <div key={key} className={styles.field}>
                  <div className={styles.fieldHeader}>
                    <label className={styles.label}>
                      {property.title || key}
                      {schema.required?.includes(key) && (
                        <span className={styles.required}>*</span>
                      )}
                    </label>
                    {property.description && (
                      <span className={styles.fieldDescription}>
                        <Info size={14} />
                        {property.description}
                      </span>
                    )}
                  </div>
                  {renderField(key, property)}
                </div>
              ))}
            </div>
          ) : (
            <div className={styles.noSchema}>
              <Info size={24} />
              <p>This plugin has no configurable settings.</p>
            </div>
          )}
        </div>

        {schema?.properties && Object.keys(schema.properties).length > 0 && (
          <div className={styles.footer}>
            <button
              className="btn btn-secondary"
              onClick={handleReset}
              disabled={!hasChanges}
            >
              <RotateCcw size={16} />
              Reset
            </button>
            <button
              className="btn btn-primary"
              onClick={handleSaveAll}
              disabled={!hasChanges || configureMutation.isPending}
            >
              <Save size={16} />
              {configureMutation.isPending ? "Saving..." : "Save Changes"}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
