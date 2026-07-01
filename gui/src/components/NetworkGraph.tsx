import { useRef, useEffect, useState } from "react";
import * as d3 from "d3";
import { colors, font } from "../theme";
import { formatMac } from "../utils/format";
import type { HostEntry } from "../types";

interface GraphNode extends d3.SimulationNodeDatum {
  id: string;
  ip: string;
  mac?: number[] | string;
  isGateway: boolean;
}

interface GraphLink extends d3.SimulationLinkDatum<GraphNode> {
  source: string | GraphNode;
  target: string | GraphNode;
}

interface TooltipData {
  ip: string;
  mac?: number[] | string;
  x: number;
  y: number;
  isGateway: boolean;
}

interface NetworkGraphProps {
  hosts: HostEntry[];
}

export function NetworkGraph({ hosts }: NetworkGraphProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const simRef = useRef<d3.Simulation<GraphNode, GraphLink> | null>(null);
  const zoomRef = useRef<d3.ZoomBehavior<SVGSVGElement, unknown> | null>(null);
  const viewportRef = useRef<SVGGElement | null>(null);

  const nodesRef = useRef<GraphNode[]>([]);
  const linksRef = useRef<GraphLink[]>([]);
  const knownRef = useRef(new Set<string>());
  const sizeRef = useRef({ w: 900, h: 600 });

  const [tooltip, setTooltip] = useState<TooltipData | null>(null);
  const [zoomLevel, setZoomLevel] = useState(1);

  useEffect(() => {
    const svgEl = svgRef.current;
    if (!svgEl) return;

    const svg = d3.select(svgEl);
    svg.selectAll("*").remove();

    const parent = svgEl.parentElement;
    const w = parent?.clientWidth ?? 900;
    const h = parent?.clientHeight ?? 600;
    sizeRef.current = { w, h };

    svg.attr("viewBox", `0 0 ${w} ${h}`);

    const defs = svg.append("defs");

    const gridPattern = defs
      .append("pattern")
      .attr("id", "network-grid")
      .attr("width", 48)
      .attr("height", 48)
      .attr("patternUnits", "userSpaceOnUse");

    gridPattern
      .append("path")
      .attr("d", "M 48 0 L 0 0 0 48")
      .attr("fill", "none")
      .attr("stroke", colors.textMuted)
      .attr("stroke-opacity", 0.08)
      .attr("stroke-width", 1);

    const gatewayGlow = defs.append("filter").attr("id", "gateway-glow");
    gatewayGlow.append("feGaussianBlur").attr("stdDeviation", 6).attr("result", "coloredBlur");
    const gatewayMerge = gatewayGlow.append("feMerge");
    gatewayMerge.append("feMergeNode").attr("in", "coloredBlur");
    gatewayMerge.append("feMergeNode").attr("in", "SourceGraphic");

    const hostGlow = defs.append("filter").attr("id", "host-glow");
    hostGlow.append("feGaussianBlur").attr("stdDeviation", 4).attr("result", "coloredBlur");
    const hostMerge = hostGlow.append("feMerge");
    hostMerge.append("feMergeNode").attr("in", "coloredBlur");
    hostMerge.append("feMergeNode").attr("in", "SourceGraphic");

    svg
      .append("rect")
      .attr("width", w)
      .attr("height", h)
      .attr("fill", "url(#network-grid)")
      .attr("pointer-events", "none");


    const viewport = svg.append("g").attr("class", "viewport");
    viewportRef.current = viewport.node();

    viewport.append("g").attr("class", "links-g");
    viewport.append("g").attr("class", "nodes-g");

    const gateway: GraphNode = {
      id: "gateway",
      ip: "Gateway",
      isGateway: true,
      x: w / 2,
      y: h / 2,
      fx: w / 2,
      fy: h / 2,
    };

    nodesRef.current = [gateway];
    linksRef.current = [];
    knownRef.current = new Set();

    const sim = d3
      .forceSimulation<GraphNode, GraphLink>(nodesRef.current)
      .force("charge", d3.forceManyBody().strength(-260).distanceMax(420))
      .force("center", d3.forceCenter(w / 2, h / 2).strength(0.045))
      .force("collision", d3.forceCollide<GraphNode>().radius((d) => (d.isGateway ? 54 : 34)).strength(0.95))
      .force(
        "link",
        d3
          .forceLink<GraphNode, GraphLink>(linksRef.current)
          .id((d) => d.id)
          .distance((d) => {
            const source = typeof d.source === "string" ? d.source : d.source.id;
            const target = typeof d.target === "string" ? d.target : d.target.id;
            return source === "gateway" || target === "gateway" ? 145 : 110;
          })
          .strength(0.28)
      )
      .alphaDecay(0.022)
      .velocityDecay(0.32);

    sim.on("tick", () => {
      svg
        .selectAll<SVGLineElement, GraphLink>(".link")
        .attr("x1", (d: any) => d.source.x)
        .attr("y1", (d: any) => d.source.y)
        .attr("x2", (d: any) => d.target.x)
        .attr("y2", (d: any) => d.target.y);

      svg
        .selectAll<SVGGElement, GraphNode>(".node-g")
        .attr("transform", (d) => `translate(${d.x},${d.y})`);
    });

    simRef.current = sim;

    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.45, 2.75])
      .filter((event) => {
        if (event.type === "dblclick") return false;
        return true;
      })
      .on("zoom", (event) => {
        viewport.attr("transform", event.transform.toString());
        setZoomLevel(event.transform.k);
      });

    zoomRef.current = zoom;
    svg.call(zoom).call(zoom.transform, d3.zoomIdentity);

    const resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) return;

      const newW = entry.contentRect.width;
      const newH = entry.contentRect.height;
      sizeRef.current = { w: newW, h: newH };

      svg.attr("viewBox", `0 0 ${newW} ${newH}`);
      svg.select("rect").attr("width", newW).attr("height", newH);
      

      const centerForce = sim.force("center") as d3.ForceCenter<GraphNode>;
      centerForce.x(newW / 2).y(newH / 2);

      const gatewayNode = nodesRef.current.find((n) => n.id === "gateway");
      if (gatewayNode) {
        gatewayNode.fx = newW / 2;
        gatewayNode.fy = newH / 2;
        gatewayNode.x = newW / 2;
        gatewayNode.y = newH / 2;
      }

      sim.alpha(0.35).restart();
    });

    if (parent) resizeObserver.observe(parent);

    return () => {
      resizeObserver.disconnect();
      sim.stop();
      svg.on(".zoom", null);
    };
  }, []);

  useEffect(() => {
    const sim = simRef.current;
    const svgEl = svgRef.current;
    if (!sim || !svgEl) return;

    const svg = d3.select(svgEl);
    const viewport = svg.select(".viewport");
    const linksGroup = viewport.select(".links-g");
    const nodesGroup = viewport.select(".nodes-g");

    const newHosts = hosts.filter((h) => !knownRef.current.has(h.ip));
    if (newHosts.length === 0) return;

    const { w, h } = sizeRef.current;

    newHosts.forEach((host) => {
      knownRef.current.add(host.ip);

      const angle = Math.random() * Math.PI * 2;
      const dist = 120 + Math.random() * 120;

      nodesRef.current.push({
        id: host.ip,
        ip: host.ip,
        mac: host.mac,
        isGateway: false,
        x: w / 2 + Math.cos(angle) * dist,
        y: h / 2 + Math.sin(angle) * dist,
      });

      linksRef.current.push({
        source: "gateway",
        target: host.ip,
      });
    });

    sim.nodes(nodesRef.current);
    (sim.force("link") as d3.ForceLink<GraphNode, GraphLink>).links(linksRef.current);
    sim.alpha(0.5).restart();

    const linkSel = linksGroup
      .selectAll<SVGLineElement, GraphLink>(".link")
      .data(
        linksRef.current,
        (d: any) =>
          `${typeof d.source === "string" ? d.source : d.source.id}-${typeof d.target === "string" ? d.target : d.target.id}`
      );

    linkSel
      .enter()
      .append("line")
      .attr("class", "link")
      .attr("stroke", colors.borderLight)
      .attr("stroke-width", 1.15)
      .attr("stroke-opacity", 0)
      .attr("stroke-linecap", "round")
      .attr("stroke-dasharray", "4 6")
      .transition()
      .duration(550)
      .attr("stroke-opacity", 0.7);

    linkSel.exit().remove();

    const nodeSel = nodesGroup
      .selectAll<SVGGElement, GraphNode>(".node-g")
      .data(nodesRef.current, (d) => d.id);

    const enter = nodeSel
      .enter()
      .append("g")
      .attr("class", "node-g")
      .style("cursor", "grab")
      .call(
        d3
          .drag<SVGGElement, GraphNode>()
          .on("start", (event, d) => {
            if (!event.active) sim.alphaTarget(0.25).restart();
            if (!d.isGateway) {
              d.fx = d.x;
              d.fy = d.y;
            }
          })
          .on("drag", (event, d) => {
            if (!d.isGateway) {
              d.fx = event.x;
              d.fy = event.y;
            }
          })
          .on("end", (event, d) => {
            if (!event.active) sim.alphaTarget(0);
            if (!d.isGateway) {
              d.fx = null;
              d.fy = null;
            }
          })
      );

    enter
      .append("circle")
      .attr("class", "halo")
      .attr("r", 0)
      .attr("fill", (d) => (d.isGateway ? colors.textSecondary : colors.accent))
      .attr("opacity", (d) => (d.isGateway ? 0.08 : 0.07))
      .attr("filter", (d) => (d.isGateway ? "url(#gateway-glow)" : "url(#host-glow)"))
      .transition()
      .duration(500)
      .attr("r", (d) => (d.isGateway ? 32 : 22));

    enter
      .append("circle")
      .attr("class", "ring")
      .attr("r", 0)
      .attr("fill", "none")
      .attr("stroke", (d) => (d.isGateway ? colors.textSecondary : colors.accent))
      .attr("stroke-width", (d) => (d.isGateway ? 1.6 : 1.2))
      .attr("stroke-opacity", 0.4)
      .transition()
      .duration(500)
      .ease(d3.easeBackOut.overshoot(1.5))
      .attr("r", (d) => (d.isGateway ? 24 : 17));

    enter
      .append("circle")
      .attr("class", "core")
      .attr("r", 0)
      .attr("fill", (d) => (d.isGateway ? colors.surface : colors.accent))
      .attr("fill-opacity", (d) => (d.isGateway ? 0.95 : 0.13))
      .attr("stroke", (d) => (d.isGateway ? colors.textSecondary : colors.accent))
      .attr("stroke-width", (d) => (d.isGateway ? 1.8 : 1.5))
      .transition()
      .duration(420)
      .ease(d3.easeBackOut.overshoot(1.8))
      .attr("r", (d) => (d.isGateway ? 15 : 10));

    enter
      .filter((d) => d.isGateway)
      .append("text")
      .attr("text-anchor", "middle")
      .attr("dy", 4)
      .attr("font-size", 11)
      .attr("font-family", font.mono)
      .attr("fill", colors.textSecondary)
      .attr("opacity", 0.95)
      .text("◆");

    enter
      .filter((d) => !d.isGateway)
      .append("circle")
      .attr("class", "host-dot")
      .attr("r", 3.2)
      .attr("fill", colors.accent)
      .attr("opacity", 0.95);

    enter
      .filter((d) => !d.isGateway)
      .append("circle")
      .attr("class", "pulse")
      .attr("r", 8)
      .attr("fill", "none")
      .attr("stroke", colors.accent)
      .attr("stroke-width", 1.4)
      .attr("stroke-opacity", 0.55)
      .transition()
      .duration(850)
      .ease(d3.easeCubicOut)
      .attr("r", 30)
      .attr("stroke-opacity", 0)
      .remove();

    enter
      .append("text")
      .attr("class", "label")
      .attr("text-anchor", "middle")
      .attr("dy", (d) => (d.isGateway ? 34 : 28))
      .attr("font-size", (d) => (d.isGateway ? 11 : 10))
      .attr("font-family", font.mono)
      .attr("font-weight", (d) => (d.isGateway ? 700 : 500))
      .attr("fill", (d) => (d.isGateway ? colors.text : colors.textSecondary))
      .attr("opacity", 0)
      .text((d) => d.ip)
      .transition()
      .delay(180)
      .duration(350)
      .attr("opacity", 0.9);

    enter
      .on("mouseenter", (event, d) => {
        setTooltip({
          ip: d.ip,
          mac: d.mac,
          x: event.clientX,
          y: event.clientY,
          isGateway: d.isGateway,
        });

        const current = d3.select(event.currentTarget);
        current
          .select<SVGCircleElement>(".core")
          .transition()
          .duration(140)
          .attr("fill-opacity", d.isGateway ? 1 : 0.22)
          .attr("stroke-width", 2.2);

        current
          .select<SVGCircleElement>(".ring")
          .transition()
          .duration(140)
          .attr("stroke-opacity", 0.9)
          .attr("r", d.isGateway ? 27 : 20);
      })
      .on("mousemove", (event, d) => {
        setTooltip((prev) =>
          prev
            ? {
                ...prev,
                ip: d.ip,
                mac: d.mac,
                x: event.clientX,
                y: event.clientY,
                isGateway: d.isGateway,
              }
            : null
        );
      })
      .on("mouseleave", (event, d) => {
        setTooltip(null);

        const current = d3.select(event.currentTarget);
        current
          .select<SVGCircleElement>(".core")
          .transition()
          .duration(140)
          .attr("fill-opacity", d.isGateway ? 0.95 : 0.13)
          .attr("stroke-width", d.isGateway ? 1.8 : 1.5);

        current
          .select<SVGCircleElement>(".ring")
          .transition()
          .duration(140)
          .attr("stroke-opacity", 0.4)
          .attr("r", d.isGateway ? 24 : 17);
      });

    nodeSel.exit().remove();
  }, [hosts]);

  const resetZoom = () => {
    const svgEl = svgRef.current;
    const zoom = zoomRef.current;
    if (!svgEl || !zoom) return;

    d3.select(svgEl)
      .transition()
      .duration(350)
      .call(zoom.transform, d3.zoomIdentity);
  };

  return (
    <div style={{ width: "100%", height: "100%", position: "relative", background: colors.bg }}>
      <div
        style={{
          position: "absolute",
          top: 14,
          right: 14,
          zIndex: 20,
          display: "flex",
          alignItems: "center",
          gap: 8,
          padding: "8px 10px",
          background: "rgba(17,24,39,0.82)",
          border: `1px solid ${colors.border}`,
          borderRadius: 10,
          backdropFilter: "blur(8px)",
          boxShadow: "0 8px 24px rgba(0,0,0,0.25)",
        }}
      >
        <span
          style={{
            fontFamily: font.mono,
            fontSize: 11,
            color: colors.textSecondary,
            minWidth: 52,
          }}
        >
          Zoom {Math.round(zoomLevel * 100)}%
        </span>
        <button
          onClick={resetZoom}
          style={{
            border: `1px solid ${colors.borderLight}`,
            background: colors.surface,
            color: colors.text,
            borderRadius: 8,
            padding: "6px 10px",
            fontFamily: font.mono,
            fontSize: 11,
            cursor: "pointer",
          }}
        >
          Reset
        </button>
      </div>

      <div
        style={{
          position: "absolute",
          left: 14,
          bottom: 14,
          zIndex: 20,
          padding: "8px 10px",
          background: "rgba(17,24,39,0.78)",
          border: `1px solid ${colors.border}`,
          borderRadius: 10,
          backdropFilter: "blur(8px)",
          fontFamily: font.mono,
          fontSize: 10,
          color: colors.textMuted,
          lineHeight: 1.7,
          pointerEvents: "none",
        }}
      >
        <div>Scroll = Zoom</div>
        <div>Drag empty space = Pan</div>
        <div>Drag node = Move</div>
      </div>

      <svg ref={svgRef} width="100%" height="100%" style={{ display: "block" }} />

      {hosts.length === 0 && (
        <div
          style={{
            position: "absolute",
            inset: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexDirection: "column",
            gap: 12,
            pointerEvents: "none",
          }}
        >
          <div
            style={{
              width: 62,
              height: 62,
              borderRadius: "50%",
              border: `1px solid ${colors.borderLight}`,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: 20,
              color: colors.textMuted,
              boxShadow: `0 0 40px ${colors.accentSoft}`,
              background: "rgba(255,255,255,0.01)",
            }}
          >
            ◎
          </div>
          <span
            style={{
              fontFamily: font.mono,
              fontSize: 12,
              color: colors.textSecondary,
              letterSpacing: 0.2,
            }}
          >
            Start an ARP scan to discover hosts
          </span>
        </div>
      )}

      {tooltip && (
        <div
          style={{
            position: "fixed",
            left: tooltip.x + 14,
            top: tooltip.y - 8,
            background: "rgba(17,24,39,0.96)",
            border: `1px solid ${colors.borderLight}`,
            borderRadius: 10,
            padding: "10px 12px",
            fontFamily: font.mono,
            fontSize: 11,
            boxShadow: "0 10px 30px rgba(0,0,0,0.38)",
            zIndex: 999,
            pointerEvents: "none",
            minWidth: 160,
            backdropFilter: "blur(8px)",
          }}
        >
          <div
            style={{
              color: tooltip.isGateway ? colors.text : colors.accent,
              fontWeight: 700,
              marginBottom: 4,
            }}
          >
            {tooltip.ip}
          </div>
          <div style={{ color: colors.textSecondary }}>
            {tooltip.isGateway ? "Core node" : formatMac(tooltip.mac)}
          </div>
        </div>
      )}
    </div>
  );
}
