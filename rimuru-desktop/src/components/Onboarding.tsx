import { useState, useEffect } from "react";
import {
  Bot,
  Layers,
  DollarSign,
  Activity,
  Sparkles,
  ChevronRight,
  ChevronLeft,
  X,
  Check
} from "lucide-react";
import { useLocalStorage } from "@/hooks/useLocalStorage";
import styles from "./Onboarding.module.css";

interface OnboardingStep {
  icon: React.ReactNode;
  title: string;
  description: string;
  features: string[];
}

const steps: OnboardingStep[] = [
  {
    icon: <Bot size={48} />,
    title: "Welcome to Rimuru",
    description: "Your unified AI agent orchestration platform. Manage, monitor, and analyze all your AI coding agents from one place.",
    features: [
      "Support for Claude Code, Copilot, Cursor, Codex, and more",
      "Real-time session tracking",
      "Automatic agent discovery",
    ],
  },
  {
    icon: <Layers size={48} />,
    title: "Agent Management",
    description: "Connect and manage multiple AI agents. Rimuru automatically detects installed agents and tracks their activity.",
    features: [
      "Auto-discovery of local AI agents",
      "Manual agent configuration",
      "Agent health monitoring",
    ],
  },
  {
    icon: <DollarSign size={48} />,
    title: "Cost Tracking",
    description: "Keep track of your AI spending across all agents. Set budgets, view trends, and optimize your usage.",
    features: [
      "Per-agent and per-model cost breakdown",
      "Daily, weekly, and monthly reports",
      "Budget alerts and limits",
    ],
  },
  {
    icon: <Activity size={48} />,
    title: "Real-time Metrics",
    description: "Monitor system performance and agent activity in real-time with detailed metrics and visualizations.",
    features: [
      "CPU and memory monitoring",
      "Active session tracking",
      "Historical metrics charts",
    ],
  },
  {
    icon: <Sparkles size={48} />,
    title: "SkillKit Integration",
    description: "Enhance your agents with skills from the SkillKit marketplace. Translate skills between different agents.",
    features: [
      "15,000+ skills available",
      "Cross-agent skill translation",
      "Plugin system for extensibility",
    ],
  },
];

interface OnboardingProps {
  onComplete: () => void;
}

export function Onboarding({ onComplete }: OnboardingProps) {
  const [currentStep, setCurrentStep] = useState(0);
  const [isExiting, setIsExiting] = useState(false);

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      handleComplete();
    }
  };

  const handlePrev = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleSkip = () => {
    handleComplete();
  };

  const handleComplete = () => {
    setIsExiting(true);
    setTimeout(() => {
      onComplete();
    }, 300);
  };

  const step = steps[currentStep];
  const isLastStep = currentStep === steps.length - 1;

  return (
    <div className={`${styles.overlay} ${isExiting ? styles.exiting : ""}`}>
      <div className={styles.modal}>
        <button
          className={styles.skipBtn}
          onClick={handleSkip}
          aria-label="Skip onboarding"
        >
          <X size={20} />
        </button>

        <div className={styles.content}>
          <div className={styles.iconWrapper}>{step.icon}</div>
          <h2 className={styles.title}>{step.title}</h2>
          <p className={styles.description}>{step.description}</p>

          <ul className={styles.features}>
            {step.features.map((feature, i) => (
              <li key={i} className={styles.feature}>
                <Check size={16} className={styles.checkIcon} />
                <span>{feature}</span>
              </li>
            ))}
          </ul>
        </div>

        <div className={styles.footer}>
          <div className={styles.dots}>
            {steps.map((_, i) => (
              <button
                key={i}
                className={`${styles.dot} ${i === currentStep ? styles.active : ""}`}
                onClick={() => setCurrentStep(i)}
                aria-label={`Go to step ${i + 1}`}
              />
            ))}
          </div>

          <div className={styles.actions}>
            {currentStep > 0 && (
              <button className="btn btn-secondary" onClick={handlePrev}>
                <ChevronLeft size={18} />
                Back
              </button>
            )}
            <button className="btn btn-primary" onClick={handleNext}>
              {isLastStep ? (
                <>
                  Get Started
                  <Sparkles size={18} />
                </>
              ) : (
                <>
                  Next
                  <ChevronRight size={18} />
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export function useOnboarding() {
  const [hasOnboarded, setHasOnboarded] = useLocalStorage("rimuru-onboarded", false);
  const [showOnboarding, setShowOnboarding] = useState(false);

  useEffect(() => {
    if (!hasOnboarded) {
      setShowOnboarding(true);
    }
  }, [hasOnboarded]);

  const completeOnboarding = () => {
    setHasOnboarded(true);
    setShowOnboarding(false);
  };

  const resetOnboarding = () => {
    setHasOnboarded(false);
    setShowOnboarding(true);
  };

  return {
    showOnboarding,
    completeOnboarding,
    resetOnboarding,
  };
}
