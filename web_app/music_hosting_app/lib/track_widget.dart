import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'shared_state.dart';

class TrackWidget extends StatefulWidget {
  final int id;
  final String authorUsername;
  final String name;
  final int cntRates;
  final int sumRates;

  const TrackWidget({
    super.key,
    required this.id,
    required this.authorUsername,
    required this.name,
    required this.cntRates,
    required this.sumRates,
  });

  @override
  _TrackWidgetState createState() => _TrackWidgetState();
}

class _TrackWidgetState extends State<TrackWidget> {
  double _calculateRating() {
    if (widget.cntRates == 0) {
      return 0.0;
    }
    return widget.sumRates / widget.cntRates;
  }

  @override
  Widget build(BuildContext context) {
    final playerData = Provider.of<PlayerData>(context);

    return Card(
      elevation: 2,
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: ListTile(
        leading: IconButton(
          icon: const Icon(Icons.play_arrow),
          onPressed: () => playerData.setCurrentTrackId(widget.id),
        ),
        title: Text(
          widget.name,
          style: const TextStyle(
            fontWeight: FontWeight.bold,
          ),
        ),
        subtitle: Text('by ${widget.authorUsername}'),
        trailing: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.star),
            const SizedBox(width: 4),
            Text(
              _calculateRating().toStringAsFixed(1),
              style: const TextStyle(
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(width: 4),
            Text('(${widget.cntRates} ratings)'),
          ],
        ),
      ),
    );
  }
}
