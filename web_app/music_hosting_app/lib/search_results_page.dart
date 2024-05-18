import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;
import 'dart:convert';

import 'track_widget.dart';

class SearchResultsPage extends StatefulWidget {
  final String query;

  const SearchResultsPage({super.key, required this.query});

  @override
  _SearchResultsPageState createState() => _SearchResultsPageState();
}

class _SearchResultsPageState extends State<SearchResultsPage> {
  late Future<List<Map<String, dynamic>>> _idsFuture;

  @override
  void initState() {
    super.initState();
    _idsFuture = _fetchIds(widget.query);
  }

  Future<List<Map<String, dynamic>>> _fetchIds(String query) async {
    final response =
        await http.get(Uri.parse('http://localhost:3000/search?query=$query'));

    if (response.statusCode == 200) {
      List<dynamic> data = jsonDecode(response.body);
      return List<Map<String, dynamic>>.from(data);
    } else {
      throw Exception('Failed to load Ids');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Search Results for "${widget.query}"'),
      ),
      body: FutureBuilder<List<Map<String, dynamic>>>(
        future: _idsFuture,
        builder: (context, snapshot) {
          if (snapshot.connectionState == ConnectionState.waiting) {
            return const Center(child: CircularProgressIndicator());
          } else if (snapshot.hasError) {
            return Center(child: Text("Error: ${snapshot.error}"));
          }

          final ids = snapshot.data ?? [];
          return ListView.builder(
            itemCount: ids.length,
            itemBuilder: (context, index) {
              // return ListTile(
              //   title: Text(ids[index]),
              // );
              return TrackWidget(
                id: ids[index]["id"],
                authorUsername: ids[index]["author_username"],
                name: ids[index]["name"],
                cntRates: ids[index]["cnt_rates"],
                sumRates: ids[index]["sum_rates"],
              );
            },
          );
        },
      ),
    );
  }
}
