import { BaseEdge, getBezierPath, type EdgeProps } from "@xyflow/react";
import { memo, useMemo } from "react";
import { bezierPoints, pointOnSampledBezier, sampleBezier } from "../graph/graph";
import { BALL_RADIUS, ballFade } from "../monitor/balls";
import { useEdgeBalls } from "../monitor/live";

/**
 * A link of the graph, with the balls that travel over it.
 *
 * The link is the curve React Flow draws between the two ports, and the balls
 * are the transactions that are on their way along it: one ball for every
 * transaction that left a block and has not reached the next one.
 *
 * The ball is drawn over the very curve the reader sees, because both come
 * from the same place: the control points of the curve and the point of it
 * where the ball is.
 *
 * The ball shrinks over the last fifth of its flight, so it vanishes as it
 * reaches the block instead of sitting on its edge. At the start it is
 * whole: the reader must see the ball leave the block the transaction came
 * from.
 *
 * The link knows nothing about the run: it asks the live layer which balls
 * are on it, and the live layer answers with the ones that are on their way
 * right now. A link with no ball is not redrawn when another link's ball
 * moves.
 */
export default memo(function GraphEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  style,
  markerEnd,
  label,
  labelStyle,
  labelBgStyle,
}: EdgeProps) {
  const [path, labelX, labelY] = useMemo(
    () =>
      getBezierPath({
        sourceX,
        sourceY,
        sourcePosition,
        targetX,
        targetY,
        targetPosition,
      }),
    [sourceX, sourceY, sourcePosition, targetX, targetY, targetPosition],
  );
  // The pre-sampled curve with arc-length table, computed once per geometry.
  const curve = useMemo(
    () =>
      sampleBezier(
        bezierPoints(
          { x: sourceX, y: sourceY },
          { x: targetX, y: targetY },
          sourcePosition,
          targetPosition,
        ),
      ),
    [sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition],
  );
  const balls = useEdgeBalls(id);
  return (
    <>
      <BaseEdge
        id={id}
        path={path}
        labelX={labelX}
        labelY={labelY}
        label={label}
        labelStyle={labelStyle}
        labelBgStyle={labelBgStyle}
        style={style}
        markerEnd={markerEnd}
      />
      {balls.map((ball, index) => {
        const point = pointOnSampledBezier(curve, ball.progress);
        return (
          <circle
            key={`${ball.xact}-${index}`}
            cx={point.x}
            cy={point.y}
            r={BALL_RADIUS * ballFade(ball.progress)}
            fill="#4f46e5"
            stroke="#ffffff"
            strokeWidth={1.5}
          />
        );
      })}
    </>
  );
});
