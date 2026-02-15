import { X } from "lucide-react";
import { useAddAgent } from "@/hooks/useAgents";
import { useForm, validators } from "@/hooks/useForm";
import { useAnimatedUnmount } from "@/hooks/useAnimatedUnmount";
import { FormField, Input, Select, FormActions, FormError } from "@/components/FormField";
import styles from "./AddAgentModal.module.css";

interface AddAgentModalProps {
  isOpen?: boolean;
  onClose: () => void;
}

const agentTypes = [
  { value: "ClaudeCode", label: "Claude Code" },
  { value: "OpenCode", label: "OpenCode" },
  { value: "Codex", label: "Codex" },
  { value: "Copilot", label: "GitHub Copilot" },
  { value: "Cursor", label: "Cursor" },
  { value: "Goose", label: "Goose" },
];

interface FormValues {
  name: string;
  agentType: string;
  configPath: string;
}

export default function AddAgentModal({ isOpen = true, onClose }: AddAgentModalProps) {
  const { shouldRender, animationState } = useAnimatedUnmount(isOpen, 200);
  const addMutation = useAddAgent();

  const form = useForm<FormValues & Record<string, unknown>>({
    initialValues: {
      name: "",
      agentType: "ClaudeCode",
      configPath: "",
    },
    validations: {
      name: [
        validators.required("Agent name is required") as never,
        validators.minLength(2, "Name must be at least 2 characters") as never,
        validators.maxLength(50, "Name must be at most 50 characters") as never,
      ],
      agentType: [validators.required("Please select an agent type") as never],
      configPath: [validators.path("Please enter a valid file path") as never],
    },
    onSubmit: async (values) => {
      try {
        await addMutation.mutateAsync({
          name: values.name as string,
          agent_type: values.agentType as string,
          config: (values.configPath as string) ? { config_dir: values.configPath } : undefined,
        });
        onClose();
      } catch (error) {
        const message = error instanceof Error ? error.message : "Failed to add agent";
        form.setFieldError("name", message);
      }
    },
    validateOnBlur: true,
    validateOnChange: false,
  });

  const nameProps = { ...form.getFieldProps("name"), value: form.values.name as string };
  const agentTypeProps = { ...form.getFieldProps("agentType"), value: form.values.agentType as string };
  const configPathProps = { ...form.getFieldProps("configPath"), value: form.values.configPath as string };

  if (!shouldRender) return null;

  return (
    <div className={styles.overlay} data-state={animationState} onClick={onClose}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <h2 className={styles.title}>Add Agent</h2>
          <button className={styles.closeBtn} onClick={onClose} aria-label="Close dialog">
            <X size={20} />
          </button>
        </div>

        <form onSubmit={form.handleSubmit} className={styles.form}>
          <FormError error={addMutation.isError ? "Failed to add agent. Please try again." : null} />

          <FormField label="Name" error={nameProps.error} required>
            <Input
              type="text"
              {...nameProps}
              placeholder="My Agent"
              autoFocus
              error={nameProps.error}
            />
          </FormField>

          <FormField label="Type" error={agentTypeProps.error} required>
            <Select
              {...agentTypeProps}
              options={agentTypes}
              error={agentTypeProps.error}
            />
          </FormField>

          <FormField
            label="Config Path"
            error={configPathProps.error}
            hint="Optional: Path to the agent's configuration file"
          >
            <Input
              type="text"
              {...configPathProps}
              placeholder="~/.claude/settings.json"
              error={configPathProps.error}
            />
          </FormField>

          <FormActions>
            <button type="button" className="btn btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button
              type="submit"
              className="btn btn-primary"
              disabled={addMutation.isPending || !(form.values.name as string)}
            >
              {addMutation.isPending ? "Adding..." : "Add Agent"}
            </button>
          </FormActions>
        </form>
      </div>
    </div>
  );
}
